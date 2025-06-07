pub mod clear_injects;
pub mod help;
pub mod inject;
pub mod set_lang;
pub mod set_mode;
pub mod status;
pub mod toggle_inject;

use async_trait::async_trait;
use frankenstein::types::Message;

use crate::service::PermissionLevel;

pub struct CommandContext {
    pub message: Message,
    pub args_raw: String,
}

impl CommandContext {
    #[allow(unused)]
    pub fn args(&self) -> Vec<&str> {
        self.args_raw.split_whitespace().collect()
    }
}

#[async_trait]
pub trait Command: Send + Sync {
    /// The primary name of the command (e.g., "help")
    fn name(&self) -> &'static str;

    /// The permission level required to execute this command.
    fn permission(&self) -> PermissionLevel {
        PermissionLevel::Admin
    }

    /// The execution logic for the command.
    async fn execute(&self, ctx: CommandContext) -> eyre::Result<()>;
}
