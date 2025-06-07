use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    service::{
        PermissionLevel,
        handlers::commands::{Command, CommandContext},
        messages::MessageManager,
        prompts::AdminDirective,
        state::ServiceState,
    },
    telegram::bot::Bot,
};

pub struct Inject {
    bot: Arc<Bot>,
    state: Arc<ServiceState>,
    message_mgr: Arc<MessageManager<'static>>,
}

impl Inject {
    pub fn new(
        bot: Arc<Bot>,
        message_mgr: Arc<MessageManager<'static>>,
        state: Arc<ServiceState>,
    ) -> Self {
        Self {
            bot,
            state,
            message_mgr,
        }
    }
}

#[async_trait]
impl Command for Inject {
    fn name(&self) -> &'static str {
        "/inject"
    }

    fn permission(&self) -> PermissionLevel {
        PermissionLevel::Admin
    }

    async fn execute(&self, ctx: CommandContext) -> eyre::Result<()> {
        let user = ctx.message.from.as_ref().unwrap();
        let args_raw = ctx.args_raw.trim_ascii();

        let chat_state = self.state.get_chat_state(ctx.message.chat.id);
        let lang = *chat_state.language.read();

        if args_raw.is_empty() {
            self.bot
                .reply(
                    ctx.message.chat.id,
                    ctx.message.message_id,
                    self.message_mgr
                        .command_usage(lang, "/inject <directive text>"),
                )
                .await?;

            return Ok(());
        }

        let directive = AdminDirective {
            author: user.first_name.clone(),
            timestamp: jiff::Timestamp::now().to_string(),
            directive_text: args_raw.to_string(),
        };
        chat_state.admin_prompt_injections.write().push(directive);

        let text = self.message_mgr.inject(lang);

        self.bot
            .reply(ctx.message.chat.id, ctx.message.message_id, text)
            .await?;

        Ok(())
    }
}
