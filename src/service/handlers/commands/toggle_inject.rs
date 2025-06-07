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

pub struct ToggleInject {
    bot: Arc<Bot>,
    state: Arc<ServiceState>,
}

impl ToggleInject {
    pub fn new(bot: Arc<Bot>, state: Arc<ServiceState>) -> Self {
        Self { bot, state }
    }
}

#[async_trait]
impl Command for ToggleInject {
    fn name(&self) -> &'static str {
        "/toggle_inject"
    }

    fn permission(&self) -> PermissionLevel {
        PermissionLevel::Admin
    }

    async fn execute(&self, ctx: CommandContext) -> eyre::Result<()> {
        let chat_state = self.state.get_chat_state(ctx.message.chat.id);

        let reply_text = {
            let mut is_active = chat_state.admin_prompt_injections_active.write();
            *is_active = !*is_active;

            let status_str = if *is_active {
                "activated"
            } else {
                "deactivated"
            };

            format!("Admin injections {status_str}.")
        };

        self.bot
            .reply(ctx.message.chat.id, ctx.message.message_id, reply_text)
            .await?;

        Ok(())
    }
}
