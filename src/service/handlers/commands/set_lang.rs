use std::{str::FromStr as _, sync::Arc};

use async_trait::async_trait;

use crate::{
    service::{
        PermissionLevel,
        handlers::commands::{Command, CommandContext},
        state::{Language, ServiceState},
    },
    telegram::bot::Bot,
};

pub struct SetLang {
    bot: Arc<Bot>,
    state: Arc<ServiceState>,
}

impl SetLang {
    pub fn new(bot: Arc<Bot>, state: Arc<ServiceState>) -> Self {
        Self { bot, state }
    }
}

#[async_trait]
impl Command for SetLang {
    fn name(&self) -> &'static str {
        "/set_lang"
    }

    fn permission(&self) -> PermissionLevel {
        PermissionLevel::Admin
    }

    async fn execute(&self, ctx: CommandContext) -> eyre::Result<()> {
        let chat_stats = self.state.get_chat_state(ctx.message.chat.id);

        let reply_text = if let Ok(new_lang) = Language::from_str(&ctx.args_raw.trim()) {
            *chat_stats.language.write() = new_lang;

            format!("Language set to {}", new_lang.as_identifier())
        } else {
            "Usage: /set_lang [en|ru]".to_string()
        };

        self.bot
            .reply(ctx.message.chat.id, ctx.message.message_id, reply_text)
            .await?;

        Ok(())
    }
}
