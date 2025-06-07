pub mod cas;
pub mod quarantine;

use std::ops::ControlFlow;

use async_trait::async_trait;
use frankenstein::types::Message;

#[async_trait]
pub trait UpdateHandler: Send + Sync {
    /// Handles the message update.
    ///
    /// The return value signals how the pipeline should proceed.
    async fn handle(&self, message: &Message) -> eyre::Result<ControlFlow<()>>;
}
