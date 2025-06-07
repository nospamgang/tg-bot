use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::Arc,
};

use arc_swap::ArcSwap;
use eyre::eyre;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::{
    service::prompts::AdminDirective,
    telegram::bot::{ChatId, UserId},
};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum Language {
    Russian,

    #[default]
    English,
}

impl Language {
    pub fn as_identifier(&self) -> &'static str {
        use Language::*;

        match self {
            Russian => "ru",
            English => "en",
        }
    }
}

impl FromStr for Language {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Language::*;

        match s.to_ascii_lowercase().as_str() {
            "ru" | "russian" => Ok(Russian),
            "en" | "english" => Ok(English),

            _ => Err(eyre!("unknown value was passed")),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Notify,

    #[default]
    Ban,
}

#[derive(Debug, Default, Clone)]
pub struct ChatState {
    /// Per-user message counter to check whether the user with the given ID left quarantine or not.
    ///
    /// The amount of messages is determined at runtime, but it's `QUARANTINED_MESSAGES_AMOUNT_INITIAL` by default.
    pub message_counter: Arc<RwLock<HashMap<UserId, u8, ahash::RandomState>>>,

    /// Determines what reaction the bot will do .
    pub working_mode: Arc<RwLock<Mode>>,

    /// The language the bot will use in the chat
    pub language: Arc<RwLock<Language>>,

    pub admin_prompt_injections_active: Arc<RwLock<bool>>,
    pub admin_prompt_injections: Arc<RwLock<Vec<AdminDirective>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveableChatState {
    pub message_counter: HashMap<UserId, u8, ahash::RandomState>,
    pub working_mode: Mode,
    pub language: Language,
    pub admin_prompt_injections_active: bool,
    pub admin_prompt_injections: Vec<AdminDirective>,
}

pub struct ServiceState {
    pub tg_chat_states: RwLock<HashMap<ChatId, ChatState, ahash::RandomState>>,
    pub cas_banned_ids: ArcSwap<HashSet<UserId, ahash::RandomState>>,
}

impl ServiceState {
    pub fn get_chat_state(&self, chat_id: ChatId) -> ChatState {
        self.tg_chat_states
            .write()
            .entry(chat_id)
            .or_default()
            .clone()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveableServiceState {
    pub tg_chat_stats: HashMap<ChatId, SaveableChatState, ahash::RandomState>,
    pub cas_banned_ids: HashSet<UserId, ahash::RandomState>,
}

impl SaveableServiceState {
    pub fn from_ref(original: &ServiceState) -> Self {
        let tg_chat_stats_lock = original.tg_chat_states.read();
        let saveable_chat_stats = tg_chat_stats_lock
            .iter()
            .map(|(chat_id, stats)| {
                let saveable_stats = SaveableChatState {
                    message_counter: stats.message_counter.read().clone(),
                    working_mode: *stats.working_mode.read(),
                    language: *stats.language.read(),
                    admin_prompt_injections_active: *stats.admin_prompt_injections_active.read(),
                    admin_prompt_injections: stats.admin_prompt_injections.read().clone(),
                };

                (*chat_id, saveable_stats)
            })
            .collect();

        Self {
            tg_chat_stats: saveable_chat_stats,
            cas_banned_ids: original.cas_banned_ids.load().as_ref().clone(),
        }
    }
}

impl From<SaveableServiceState> for ServiceState {
    fn from(value: SaveableServiceState) -> Self {
        let tg_chat_states = value
            .tg_chat_stats
            .into_iter()
            .map(|(id, s)| {
                let stats = ChatState {
                    message_counter: Arc::new(RwLock::new(s.message_counter)),
                    working_mode: Arc::new(RwLock::new(s.working_mode)),
                    language: Arc::new(RwLock::new(s.language)),
                    admin_prompt_injections_active: Arc::new(RwLock::new(
                        s.admin_prompt_injections_active,
                    )),
                    admin_prompt_injections: Arc::new(RwLock::new(s.admin_prompt_injections)),
                };

                (id, stats)
            })
            .collect();

        Self {
            tg_chat_states: RwLock::new(tg_chat_states),
            cas_banned_ids: ArcSwap::new(Arc::new(value.cas_banned_ids)),
        }
    }
}

impl Default for ServiceState {
    fn default() -> Self {
        Self {
            tg_chat_states: Default::default(),
            cas_banned_ids: Default::default(),
        }
    }
}
