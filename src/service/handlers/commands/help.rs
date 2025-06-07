use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    service::{
        PermissionLevel,
        handlers::commands::{Command, CommandContext},
        messages::MessageManager,
        state::ServiceState,
    },
    telegram::bot::Bot,
};

pub struct Help {
    bot: Arc<Bot>,
    message_mgr: Arc<MessageManager<'static>>,
    state: Arc<ServiceState>,
}

impl Help {
    pub fn new(
        bot: Arc<Bot>,
        message_mgr: Arc<MessageManager<'static>>,
        state: Arc<ServiceState>,
    ) -> Self {
        Self {
            bot,
            message_mgr,
            state,
        }
    }
}

#[async_trait]
impl Command for Help {
    fn name(&self) -> &'static str {
        "/help"
    }

    fn permission(&self) -> crate::service::PermissionLevel {
        PermissionLevel::Public
    }

    async fn execute(&self, ctx: CommandContext) -> eyre::Result<()> {
        let chat_stats = self.state.get_chat_state(ctx.message.chat.id);
        let lang = *chat_stats.language.read();
        let help_text = self.message_mgr.help(lang);

        self.bot
            .reply(ctx.message.chat.id, ctx.message.message_id, help_text)
            .await?;

        Ok(())
    }
}
