mod handlers;
mod messages;
mod prompts;
pub mod service;
mod state;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionLevel {
    Public,
    Admin,
}
