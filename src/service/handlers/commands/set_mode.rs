use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    service::{
        PermissionLevel,
        handlers::commands::{Command, CommandContext},
        state::{Mode, ServiceState},
    },
    telegram::bot::Bot,
};

pub struct SetMode {
    bot: Arc<Bot>,
    state: Arc<ServiceState>,
}

impl SetMode {
    pub fn new(bot: Arc<Bot>, state: Arc<ServiceState>) -> Self {
        Self { bot, state }
    }
}

#[async_trait]
impl Command for SetMode {
    fn name(&self) -> &'static str {
        "/set_mode"
    }

    fn permission(&self) -> PermissionLevel {
        PermissionLevel::Admin
    }

    async fn execute(&self, ctx: CommandContext) -> eyre::Result<()> {
        let chat_stats = self.state.get_chat_state(ctx.message.chat.id);

        let reply_text = match ctx.args_raw.trim().to_lowercase().as_str() {
            "ban" => {
                *chat_stats.working_mode.write() = Mode::Ban;

                "Mode set to: Ban"
            }
            "notify" => {
                *chat_stats.working_mode.write() = Mode::Notify;

                "Mode set to: Notify"
            }
            _ => "Usage: /set_mode [ban|notify]",
        };
        self.bot
            .reply(ctx.message.chat.id, ctx.message.message_id, reply_text)
            .await?;
        Ok(())
    }
}
