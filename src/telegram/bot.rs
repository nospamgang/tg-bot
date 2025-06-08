use std::sync::Arc;

use eyre::Context;
use frankenstein::{
    AsyncTelegramApi, ParseMode,
    client_reqwest::Bot as FrankenBot,
    methods::{
        BanChatMemberParams, DeleteMessageParams, GetChatAdministratorsParams, GetUpdatesParams,
        SendMessageParams, SetWebhookParams,
    },
    types::{AllowedUpdate, ChatMember, Message, ReplyParameters},
    updates::Update,
};
use reqwest::Client as ReqwestClient;
use tracing::debug;

// I don't care about abstractions that much regarding that bot so they will leak, I think
// the only reason for this particular file is to simplify the frankenstein's API

pub type ChatId = i64;
pub type UserId = u64;
pub type MessageId = i32;

pub struct Bot {
    api: FrankenBot,
}

impl Bot {
    pub fn new(token: impl AsRef<str>, http_client: Arc<ReqwestClient>) -> Self {
        debug!("Initializing bot (polling)...");

        let token = token.as_ref();

        let api = FrankenBot::builder()
            .api_url(format!("{}{}", frankenstein::BASE_API_URL, token))
            .client(Arc::unwrap_or_clone(http_client))
            .build();

        Self { api }
    }

    pub async fn set_webhook(
        &self,
        allowed_updates: Vec<AllowedUpdate>,
        url: impl Into<String>,
        secret_token: Option<String>,
    ) -> eyre::Result<()> {
        self.api
            .set_webhook(
                &SetWebhookParams::builder()
                    .url(url)
                    .maybe_secret_token(secret_token)
                    .allowed_updates(allowed_updates)
                    .build(),
            )
            .await
            .wrap_err("unable to set webhook")?;

        Ok(())
    }

    pub async fn updates(&self, params: GetUpdatesParams) -> eyre::Result<Vec<Update>> {
        let resp = self
            .api
            .get_updates(&params)
            .await
            .wrap_err("unable to send request")?;

        Ok(resp.result)
    }

    pub async fn send_message(
        &self,
        chat_id: ChatId,
        text: impl Into<String>,
    ) -> eyre::Result<Message> {
        let resp = self
            .api
            .send_message(
                &SendMessageParams::builder()
                    .chat_id(chat_id)
                    .text(text)
                    .parse_mode(ParseMode::Html)
                    .build(),
            )
            .await
            .wrap_err("unable to send request")?;

        Ok(resp.result)
    }

    pub async fn reply(
        &self,
        chat_id: ChatId,
        reply_to_message_id: MessageId,
        text: impl Into<String>,
    ) -> eyre::Result<Message> {
        let resp = self
            .api
            .send_message(
                &SendMessageParams::builder()
                    .chat_id(chat_id)
                    .text(text)
                    .parse_mode(ParseMode::MarkdownV2)
                    .reply_parameters(
                        ReplyParameters::builder()
                            .message_id(reply_to_message_id)
                            .build(),
                    )
                    .build(),
            )
            .await
            .wrap_err("unable to send request")?;

        Ok(resp.result)
    }

    pub async fn delete_message(&self, chat_id: ChatId, message_id: MessageId) -> eyre::Result<()> {
        self.api
            .delete_message(
                &DeleteMessageParams::builder()
                    .chat_id(chat_id)
                    .message_id(message_id)
                    .build(),
            )
            .await
            .wrap_err("unable to delete message")?;

        Ok(())
    }

    pub async fn ban_user(&self, chat_id: ChatId, user_id: UserId) -> eyre::Result<()> {
        self.api
            .ban_chat_member(
                &BanChatMemberParams::builder()
                    .chat_id(chat_id)
                    .user_id(user_id)
                    .build(),
            )
            .await
            .wrap_err("unable to ban user")?;

        Ok(())
    }

    pub async fn get_chat_admins(&self, chat_id: ChatId) -> eyre::Result<Vec<ChatMember>> {
        let resp = self
            .api
            .get_chat_administrators(
                &GetChatAdministratorsParams::builder()
                    .chat_id(chat_id)
                    .build(),
            )
            .await
            .wrap_err("unable to get chat administrators")?;

        Ok(resp.result)
    }
}
