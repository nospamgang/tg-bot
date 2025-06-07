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

pub struct Status {
    bot: Arc<Bot>,
    state: Arc<ServiceState>,
}

impl Status {
    pub fn new(bot: Arc<Bot>, state: Arc<ServiceState>) -> Self {
        Self { bot, state }
    }
}

#[async_trait]
impl Command for Status {
    fn name(&self) -> &'static str {
        "/status"
    }

    fn permission(&self) -> PermissionLevel {
        PermissionLevel::Admin
    }

    async fn execute(&self, ctx: CommandContext) -> eyre::Result<()> {
        let chat_state = self.state.get_chat_state(ctx.message.chat.id);
        let status_text = format!("{:#?}", chat_state);

        self.bot
            .reply(
                ctx.message.chat.id,
                ctx.message.message_id,
                format!("{status_text}"),
            )
            .await?;

        Ok(())
    }
}
