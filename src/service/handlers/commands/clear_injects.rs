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

pub struct ClearInjects {
    bot: Arc<Bot>,
    state: Arc<ServiceState>,
    message_mgr: Arc<MessageManager<'static>>,
}

impl ClearInjects {
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

        let chat_state = self.state.get_chat_state(ctx.message.chat.id);
        let lang = *chat_state.language.read();
        let text = self.message_mgr.clear_injects(lang);

        self.bot
            .reply(ctx.message.chat.id, ctx.message.message_id, text)
            .await?;

        Ok(())
    }
}
