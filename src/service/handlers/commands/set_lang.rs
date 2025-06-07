use std::{str::FromStr as _, sync::Arc};

use async_trait::async_trait;

use crate::{
    service::{
        PermissionLevel,
        handlers::commands::{Command, CommandContext},
        messages::MessageManager,
        state::{Language, ServiceState},
    },
    telegram::bot::Bot,
};

pub struct SetLang {
    bot: Arc<Bot>,
    state: Arc<ServiceState>,
    message_mgr: Arc<MessageManager<'static>>,
}

impl SetLang {
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
impl Command for SetLang {
    fn name(&self) -> &'static str {
        "/set_lang"
    }

    fn permission(&self) -> PermissionLevel {
        PermissionLevel::Admin
    }

    async fn execute(&self, ctx: CommandContext) -> eyre::Result<()> {
        let chat_state = self.state.get_chat_state(ctx.message.chat.id);

        let reply_text = if let Ok(new_lang) = Language::from_str(&ctx.args_raw.trim()) {
            *chat_state.language.write() = new_lang;

            self.message_mgr
                .set_lang(new_lang, new_lang.as_identifier())
        } else {
            let lang = *chat_state.language.read();

            self.message_mgr.command_usage(lang, "/set_lang [en|ru]")
        };

        self.bot
            .reply(ctx.message.chat.id, ctx.message.message_id, reply_text)
            .await?;

        Ok(())
    }
}
