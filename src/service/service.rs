use std::{collections::HashMap, ops::ControlFlow, sync::Arc, time::Duration};

use fjall::{Keyspace, PartitionCreateOptions, PartitionHandle, PersistMode};
use frankenstein::{
    methods::GetUpdatesParams,
    updates::{Update, UpdateContent},
};
use minijinja::Environment;
use reqwest::Client as ReqwestClient;
use tracing::{debug, error, info};

use crate::{
    ai::provider::Provider,
    config::Config,
    service::{
        PermissionLevel,
        handlers::{
            commands::{
                Command, CommandContext, clear_injects::ClearInjects, help::Help, inject::Inject,
                set_lang::SetLang, set_mode::SetMode, status::Status, toggle_inject::ToggleInject,
            },
            updates::{UpdateHandler, cas::CasBan, quarantine::Quarantine},
        },
        messages::MessageManager,
        prompts::PromptManager,
        state::{SaveableServiceState, ServiceState},
    },
    telegram::{
        bot::{Bot, ChatId, UserId},
        cas::Client as CasClient,
    },
};

/// Telegram bot updates fetching interval
const POLL_INTERVAL: Duration = Duration::from_secs(3);

/// How often to save the state to the database
const STATE_SAVE_INTERVAL: Duration = Duration::from_secs(30);

/// How often to refresh the CAS banned list
const CAS_REFRESH_INTERVAL: Duration = Duration::from_secs(60 * 60); // 1 hour

const STATE_PARTITION_NAME: &str = "service_state";
const STATE_PARTITION_KEY: &str = "main";

pub struct Service {
    bot: Arc<Bot>,
    db: Arc<Keyspace>,
    state_partition: Arc<PartitionHandle>,
    cas: Arc<CasClient>,
    state: Arc<ServiceState>,

    command_registry: HashMap<String, Box<dyn Command>, ahash::RandomState>,
    update_handlers: Vec<Box<dyn UpdateHandler>>,
}

impl Service {
    pub fn new(
        config: Arc<Config>,
        http_client: Arc<ReqwestClient>,
        ai_provider: Arc<dyn Provider + Send + Sync>,
        db: Arc<Keyspace>,
    ) -> Self {
        info!("Initializing service with the bot in polling mode...");

        // let db = Arc::new(
        //     FjallConfig::new(config.db_file())
        //         .open()
        //         .expect("failed to open database"),
        // );
        let state_partition = Arc::new(
            db.open_partition(STATE_PARTITION_NAME, PartitionCreateOptions::default())
                .expect("failed to open state partition"),
        );

        let state = Arc::new(
            if let Ok(Some(bytes)) = state_partition.get(STATE_PARTITION_KEY) {
                info!("Loading state from DB...");
                serde_json::from_slice::<SaveableServiceState>(&bytes)
                    .map(Into::into)
                    .unwrap_or_else(|e| {
                        error!("Failed to parse state from DB: {e}. Using default.");

                        ServiceState::default()
                    })
            } else {
                ServiceState::default()
            },
        );

        let bot = Arc::new(Bot::new(config.tg_bot_api_key(), http_client.clone()));
        let cas = Arc::new(CasClient::new(http_client.clone()));

        let mut jinja_env = Environment::new();
        minijinja_embed::load_templates!(&mut jinja_env);
        let jinja_env = Arc::new(jinja_env);
        let prompt_mgr = Arc::new(PromptManager::new(jinja_env.clone()));
        let message_mgr = Arc::new(MessageManager::new(jinja_env));

        let commands: Vec<Box<dyn Command + 'static>> = vec![
            Box::new(Help::new(bot.clone(), message_mgr.clone(), state.clone())),
            Box::new(SetMode::new(
                bot.clone(),
                message_mgr.clone(),
                state.clone(),
            )),
            Box::new(SetLang::new(
                bot.clone(),
                message_mgr.clone(),
                state.clone(),
            )),
            Box::new(Inject::new(bot.clone(), message_mgr.clone(), state.clone())),
            Box::new(ToggleInject::new(
                bot.clone(),
                message_mgr.clone(),
                state.clone(),
            )),
            Box::new(ClearInjects::new(
                bot.clone(),
                message_mgr.clone(),
                state.clone(),
            )),
            Box::new(Status::new(bot.clone(), state.clone())),
        ];

        let mut command_registry = HashMap::default();
        for cmd in commands {
            command_registry.insert(cmd.name().to_string(), cmd);
        }

        // order matters here
        // maybe TODO: implement priority queue?
        let update_handlers: Vec<Box<dyn UpdateHandler>> = vec![
            Box::new(CasBan::new(bot.clone(), state.clone(), message_mgr.clone())),
            Box::new(Quarantine::new(
                bot.clone(),
                ai_provider,
                state.clone(),
                prompt_mgr.clone(),
                message_mgr.clone(),
            )),
        ];

        Self {
            bot,
            db,
            state_partition,
            state,
            cas,
            command_registry: command_registry,
            update_handlers: update_handlers,
        }
    }

    pub async fn start(self: Arc<Self>) -> eyre::Result<()> {
        let s_db = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(STATE_SAVE_INTERVAL);

            loop {
                interval.tick().await;

                if let Err(e) = s_db.save_state_to_db().await {
                    error!("Failed to save state to DB: {e}");
                }
            }
        });

        let s_cas = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(CAS_REFRESH_INTERVAL);

            loop {
                interval.tick().await;
                if let Err(e) = s_cas.update_cas_banned_ids().await {
                    error!("Failed to update CAS list: {e}");
                }
            }
        });

        info!("Starting main polling loop...");

        self.start_tg_update_handler().await
    }

    async fn save_state_to_db(&self) -> eyre::Result<()> {
        let state_snapshot = SaveableServiceState::from_ref(&self.state);
        let serialized = serde_json::to_vec(&state_snapshot)?;
        self.state_partition
            .insert(STATE_PARTITION_KEY, serialized)?;

        let db_clone = self.db.clone();
        tokio::task::spawn_blocking(move || db_clone.persist(PersistMode::SyncAll)).await??;

        debug!("Service state saved to DB.");
        Ok(())
    }

    pub async fn update_cas_banned_ids(&self) -> eyre::Result<()> {
        info!("Fetching full CAS banned list...");
        let new_list = self.cas.fetch_full_list().await?;

        self.state.cas_banned_ids.store(Arc::new(new_list));

        Ok(())
    }

    async fn start_tg_update_handler(self: Arc<Self>) -> eyre::Result<()> {
        let mut offset = 0;

        loop {
            let params = GetUpdatesParams::builder()
                .offset(offset)
                .timeout(1)
                .build();

            match self.bot.updates(params).await {
                Ok(updates) => {
                    let max_id = updates.iter().map(|u| u.update_id).max();

                    for update in updates {
                        let s = self.clone();
                        tokio::spawn(async move {
                            if let Err(e) = s.dispatch_update(update).await {
                                error!("Error handling update: {e}");
                            }
                        });
                    }

                    if let Some(max_id) = max_id {
                        // .offset accepts only i64
                        offset = max_id as i64 + 1;
                    }
                }
                Err(e) => {
                    error!("Failed to get updates: {e}");
                }
            }

            tokio::time::sleep(POLL_INTERVAL).await;
        }
    }

    async fn dispatch_update(self: Arc<Self>, update: Update) -> eyre::Result<()> {
        let msg = if let UpdateContent::Message(msg) = update.content {
            msg
        } else {
            return Ok(()); // non-message update (ok, but we don't handle them)
        };

        let user = match &msg.from {
            Some(user) if !user.is_bot => user,

            _ => return Ok(()),
        };

        for handler in &self.update_handlers {
            if handler.handle(&msg).await? == ControlFlow::Break(()) {
                return Ok(());
            }
        }

        if let Some(text) = &msg.text {
            if text.starts_with('/') {
                let (command_name, args_raw) = text.split_once(' ').unwrap_or((text, ""));
                if let Some(command) = self.command_registry.get(command_name) {
                    let has_permission = match command.permission() {
                        PermissionLevel::Public => true,
                        PermissionLevel::Admin => {
                            self.is_user_admin(msg.chat.id, user.id as UserId).await?
                        }
                    };

                    if has_permission {
                        let ctx = CommandContext {
                            message: *msg.clone(),
                            args_raw: args_raw.to_string(),
                        };

                        return command.execute(ctx).await;
                    }
                }
            }
        }

        Ok(())
    }

    async fn is_user_admin(&self, chat_id: ChatId, user_id: UserId) -> eyre::Result<bool> {
        use frankenstein::types::ChatMember::*;

        let admins = self.bot.get_chat_admins(chat_id).await?;

        Ok(admins.iter().any(|m| match m {
            Creator(u) => u.user.id as UserId == user_id,
            Administrator(u) => u.user.id as UserId == user_id,

            _ => false,
        }))
    }
}
