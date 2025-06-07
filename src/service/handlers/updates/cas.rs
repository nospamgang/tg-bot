use std::{ops::ControlFlow, sync::Arc};

use async_trait::async_trait;
use frankenstein::types::Message;
use tracing::info;

use crate::{
    service::{handlers::updates::UpdateHandler, messages::MessageManager, state::ServiceState},
    telegram::bot::{Bot, UserId},
};

pub struct CasBan {
    bot: Arc<Bot>,
    state: Arc<ServiceState>,
    message_mgr: Arc<MessageManager<'static>>,
}

impl CasBan {
    pub fn new(
        bot: Arc<Bot>,
        state: Arc<ServiceState>,
        message_mgr: Arc<MessageManager<'static>>,
    ) -> Self {
        Self {
            bot,
            state,
            message_mgr,
        }
    }
}

#[async_trait]
impl UpdateHandler for CasBan {
    async fn handle(&self, message: &Message) -> eyre::Result<ControlFlow<()>> {
        let user_id = message.from.as_ref().unwrap().id as UserId;

        if self.state.cas_banned_ids.load().contains(&user_id) {
            info!("CAS-banning user {} in chat {}", user_id, message.chat.id);

            self.bot.ban_user(message.chat.id, user_id).await?;
            let _ = self
                .bot
                .delete_message(message.chat.id, message.message_id)
                .await?;

            let chat_state = self.state.get_chat_state(message.chat.id);
            let lang = *chat_state.language.read();

            let reply = self.message_mgr.antispam_notification(
                lang,
                user_id,
                "CAS Ban",
                self.message_mgr.cas(lang),
            );

            self.bot.send_message(message.chat.id, reply).await?;

            return Ok(ControlFlow::Break(()));
        }

        Ok(ControlFlow::Continue(()))
    }
}
