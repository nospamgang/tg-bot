use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    service::{
        PermissionLevel,
        handlers::commands::{Command, CommandContext},
        state::ServiceState,
    },
    telegram::bot::Bot,
};

pub struct ClearInjects {
    bot: Arc<Bot>,
    state: Arc<ServiceState>,
}

impl ClearInjects {
    pub fn new(bot: Arc<Bot>, state: Arc<ServiceState>) -> Self {
        Self { bot, state }
    }
}

#[async_trait]
impl Command for ClearInjects {
    fn name(&self) -> &'static str {
        "/clear_injects"
    }

    fn permission(&self) -> PermissionLevel {
        PermissionLevel::Admin
    }

    async fn execute(&self, ctx: CommandContext) -> eyre::Result<()> {
        let chat_state = self.state.get_chat_state(ctx.message.chat.id);
        chat_state.admin_prompt_injections.write().clear();

        self.bot
            .reply(
                ctx.message.chat.id,
                ctx.message.message_id,
                "All admin directives cleared.".to_string(),
            )
            .await?;

        Ok(())
    }
}
