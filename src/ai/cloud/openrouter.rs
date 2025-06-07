use std::sync::Arc;

use arc_swap::ArcSwap;
use async_trait::async_trait;
use const_format::formatcp;
use eyre::{Context, OptionExt};
use serde::Deserialize;
use serde_json::json;

use crate::ai::provider::{ChatMessage, Provider};

pub const DEFAULT_MODEL: &str = "google/gemini-2.5-flash-preview-05-20";

pub struct Openrouter {
    token: Arc<String>,
    active_model_name: ArcSwap<String>,
    http_client: Arc<reqwest::Client>,
}

impl Openrouter {
    pub const BASE_URL: &str = "https://openrouter.ai/api";

    pub fn new(
        http_client: Arc<reqwest::Client>,
        token: Arc<String>,
        model_name: Arc<String>,
    ) -> Self {
        Self {
            token,
            active_model_name: ArcSwap::new(model_name),
            http_client,
        }
    }

    #[allow(unused)]
    fn change_model(&self, model_name: Arc<String>) {
        self.active_model_name.store(model_name);
    }
}

#[async_trait]
impl Provider for Openrouter {
    fn token(&self) -> Arc<String> {
        self.token.clone()
    }

    async fn chat(&self, messages: Vec<ChatMessage>) -> eyre::Result<String> {
        let model = self.active_model_name.load();
        let model = model.as_ref();

        let req = self
            .http_client
            .post(formatcp!("{}/v1/chat/completions", Openrouter::BASE_URL))
            .bearer_auth(&self.token())
            .json(&json!(
                {
                    "model": model,
                    "provider": {
                        "sort": "price",
                    },
                    "messages": messages,
                    "temperature": 0.8,
                    "reasoning": {
                        "exclude": true,
                    },
                }
            ))
            .send()
            .await
            .wrap_err("error when sending a request")?
            .error_for_status()
            .wrap_err("HTTP error when doing a request")?;

        let resp = req
            .json::<ChatResponse>()
            .await
            .wrap_err("unable to parse the chat response")?;

        Ok(resp
            .choices
            .first()
            .ok_or_eyre("no choice messages in the response")?
            .message
            .text()
            .clone())
    }
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChatMessage,
}
