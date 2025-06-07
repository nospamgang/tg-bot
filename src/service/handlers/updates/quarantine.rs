use std::{ops::ControlFlow, sync::Arc};

use async_trait::async_trait;
use eyre::Context as _;
use frankenstein::types::{Message, User};
use tracing::{debug, info};

use crate::{
    ai::provider::{ChatMessage, Provider},
    service::{
        handlers::updates::UpdateHandler,
        messages::MessageManager,
        prompts::{
            MessageAnalysisContext, PromptManager,
            report::{self, AnalysisReport},
        },
        state::{ChatState, Language, Mode, ServiceState},
    },
    telegram::bot::{Bot, UserId},
};

/// The amount of messages that will be checked
const QUARANTINED_MESSAGES_AMOUNT_INITIAL: usize = 3;

pub struct Quarantine {
    bot: Arc<Bot>,
    ai_provider: Arc<dyn Provider>,
    state: Arc<ServiceState>,
    prompt_mgr: Arc<PromptManager<'static>>,
    message_mgr: Arc<MessageManager<'static>>,
}

impl Quarantine {
    pub fn new(
        bot: Arc<Bot>,
        ai_provider: Arc<dyn Provider>,
        state: Arc<ServiceState>,
        prompt_mgr: Arc<PromptManager<'static>>,
        message_mgr: Arc<MessageManager<'static>>,
    ) -> Self {
        Self {
            bot,
            ai_provider,
            state,
            prompt_mgr,
            message_mgr,
        }
    }

    async fn analyze_message(
        &self,
        text: &str,
        user: &User,
        chat_state: &ChatState,
        lang: Language,
    ) -> eyre::Result<AnalysisReport> {
        let system_prompt = self.prompt_mgr.moderator_system_prompt();
        let mut messages = vec![ChatMessage::System(system_prompt)];

        if *chat_state.admin_prompt_injections_active.read() {
            let directives = chat_state.admin_prompt_injections.read();
            if !directives.is_empty() {
                let admin_prompt = self.prompt_mgr.admin_prompt_inject(
                    &crate::service::prompts::AdminInjectionContext {
                        recent_admin_directives: directives.clone(),
                    },
                );
                messages.push(ChatMessage::User(admin_prompt));
            }
        }

        let check_prompt = self
            .prompt_mgr
            .check_message_prompt(&MessageAnalysisContext {
                original_message_text: text.to_string(),
                user_account_name: format!(
                    "{} {}",
                    user.first_name,
                    user.last_name.as_deref().unwrap_or("")
                ),
                user_account_description: user.username.clone(),
                user_account_join_date: None,
                output_language: lang.as_identifier().to_string(),
            });

        messages.push(ChatMessage::User(check_prompt));

        let response_text = self.ai_provider.chat(messages).await?;
        let clean_json = response_text
            .trim()
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim();

        serde_json::from_str(clean_json)
            .wrap_err_with(|| format!("Failed to parse AI response. Raw: '{response_text}'"))
    }
}

#[async_trait]
impl UpdateHandler for Quarantine {
    async fn handle(&self, message: &Message) -> eyre::Result<ControlFlow<()>> {
        let user = message.from.as_ref().unwrap();
        let user_id = user.id as UserId;
        let chat_state = self.state.get_chat_state(message.chat.id);

        let is_under_quarantine = {
            let mut msg_counter = chat_state.message_counter.write();
            let count = msg_counter.entry(user_id).or_insert(0);

            if *count < QUARANTINED_MESSAGES_AMOUNT_INITIAL as _ {
                *count += 1;

                debug!(
                    "User {} in chat {} is in quarantine (message {}/{})",
                    user_id, message.chat.id, *count, QUARANTINED_MESSAGES_AMOUNT_INITIAL
                );

                true
            } else {
                msg_counter.remove(&user_id);

                false
            }
        };

        if !is_under_quarantine {
            return Ok(ControlFlow::Continue(()));
        }

        let lang = *chat_state.language.read();
        let analysis_result = self
            .analyze_message(
                &message.text.clone().unwrap_or_default(),
                &user,
                &chat_state,
                lang,
            )
            .await;

        if let Ok(report) = analysis_result {
            if report.assessment_outcome == report::AssessmentOutcome::Flag {
                info!(
                    "AI flagged message from user {}. Action: {:?}",
                    user_id, report.suggested_action
                );

                let should_ban = *chat_state.working_mode.read() == Mode::Ban;

                if should_ban {
                    self.bot.ban_user(message.chat.id, user_id).await?;
                }

                let _ = self
                    .bot
                    .delete_message(message.chat.id, message.message_id)
                    .await;

                let reply = self.message_mgr.antispam_notification(
                    lang,
                    user_id,
                    &report
                        .primary_reason
                        .unwrap_or_else(|| "AI Spam Filter".into()),
                    &report
                        .detailed_reasoning
                        .unwrap_or_else(|| "No details provided.".into()),
                );
                self.bot.send_message(message.chat.id, reply).await?;

                return Ok(ControlFlow::Break(()));
            }
        }

        Ok(ControlFlow::Continue(()))
    }
}
