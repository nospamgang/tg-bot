use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    service::{
        PermissionLevel,
        handlers::commands::{Command, CommandContext},
        prompts::AdminDirective,
        state::ServiceState,
    },
    telegram::bot::Bot,
};

pub struct Inject {
    bot: Arc<Bot>,
    state: Arc<ServiceState>,
}

impl Inject {
    pub fn new(bot: Arc<Bot>, state: Arc<ServiceState>) -> Self {
        Self { bot, state }
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

        if args_raw.is_empty() {
            self.bot
                .reply(
                    ctx.message.chat.id,
                    ctx.message.message_id,
                    "Usage: /inject <directive text>",
                )
                .await?;

            return Ok(());
        }

        let directive = AdminDirective {
            author: user.first_name.clone(),
            timestamp: jiff::Timestamp::now().to_string(),
            directive_text: args_raw.to_string(),
        };
        let chat_state = self.state.get_chat_state(ctx.message.chat.id);
        chat_state.admin_prompt_injections.write().push(directive);

        self.bot
            .reply(
                ctx.message.chat.id,
                ctx.message.message_id,
                "Directive added",
            )
            .await?;

        Ok(())
    }
}
