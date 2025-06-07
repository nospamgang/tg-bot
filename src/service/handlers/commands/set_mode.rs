use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    service::{
        PermissionLevel,
        handlers::commands::{Command, CommandContext},
        messages::MessageManager,
        state::{Mode, ServiceState},
    },
    telegram::bot::Bot,
};

pub struct SetMode {
    bot: Arc<Bot>,
    state: Arc<ServiceState>,
    message_mgr: Arc<MessageManager<'static>>,
}

impl SetMode {
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
impl Command for SetMode {
    fn name(&self) -> &'static str {
        "/set_mode"
    }

    fn permission(&self) -> PermissionLevel {
        PermissionLevel::Admin
    }

    async fn execute(&self, ctx: CommandContext) -> eyre::Result<()> {
        let chat_state = self.state.get_chat_state(ctx.message.chat.id);
        let lang = *chat_state.language.read();

        let reply_text = match ctx.args_raw.trim().to_lowercase().as_str() {
            "ban" => {
                *chat_state.working_mode.write() = Mode::Ban;

                self.message_mgr.set_mode(lang, "Ban")
            }
            "notify" => {
                *chat_state.working_mode.write() = Mode::Notify;

                self.message_mgr.set_mode(lang, "Notify")
            }
            _ => self
                .message_mgr
                .command_usage(lang, "/set_mode [ban|notify]"),
        };

        self.bot
            .reply(ctx.message.chat.id, ctx.message.message_id, reply_text)
            .await?;

        Ok(())
    }
}
