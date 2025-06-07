use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// TODO: add a way to provide the default model name but in a way that's not tied to OpenAI API
//       a new struct Model probably would be required

#[async_trait]
pub trait Provider: Send + Sync {
    /// Access token
    fn token(&self) -> Arc<String>;

    /// "chat compeltion" in the context of OpenAI API
    async fn chat(&self, messages: Vec<ChatMessage>) -> eyre::Result<String>;
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "role", content = "message")]
pub enum ChatMessage {
    System(String),
    Developer(String),
    User(String),
    Assistant(String),
    Tool(String),
}

impl ChatMessage {
    pub fn text(&self) -> &String {
        use ChatMessage::*;

        match self {
            System(s) | Developer(s) | User(s) | Assistant(s) | Tool(s) => &s,
        }
    }
}
