#![expect(unused)]

use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use clap::Parser;
use const_format::formatcp;
use eyre::{OptionExt, eyre};
use url::Url;

use crate::dotenv;

const TELEGRAM_BOT_API_KEY_ENV: &str = "TELEGRAM_BOT_API_KEY";
const OPENROUTER_API_KEY_ENV: &str = "OPENROUTER_API_KEY";

/// A secret token to be sent in a header 'X-Telegram-Bot-Api-Secret-Token' in every webhook request.
const TELEGRAM_WEBHOOK_SECRET_ENV: &str = "TELEGRAM_WEBHOOK_SECRET";

#[derive(Debug)]
pub struct Config {
    tg_bot_api_key: String,
    openrouter_api_key: String,
    db_file: PathBuf,
    default_model: String,

    webhook_url: Option<Url>,
    webhook_listen_addr: Option<SocketAddr>,
    webhook_secret_token: Option<String>,
}

impl Config {
    fn is_tg_key_not_set(&self) -> bool {
        self.tg_bot_api_key == String::default()
    }

    fn is_openrouter_not_set(&self) -> bool {
        self.openrouter_api_key == String::default()
    }

    fn is_db_file_not_set(&self) -> bool {
        self.db_file == PathBuf::default()
    }

    pub fn is_webhook_active(&self) -> bool {
        self.webhook_url.is_some()
    }

    pub fn tg_bot_api_key(&self) -> &str {
        &self.tg_bot_api_key
    }

    pub fn openrouter_api_key(&self) -> &str {
        &self.openrouter_api_key
    }

    pub fn db_file(&self) -> &PathBuf {
        &self.db_file
    }

    pub fn webhook_url(&self) -> Option<&Url> {
        self.webhook_url.as_ref()
    }

    pub fn webhook_listen_addr(&self) -> Option<&SocketAddr> {
        self.webhook_listen_addr.as_ref()
    }

    pub fn webhook_secret_token(&self) -> Option<&str> {
        self.webhook_secret_token.as_deref()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tg_bot_api_key: Default::default(),
            openrouter_api_key: Default::default(),
            db_file: Default::default(),
            default_model: crate::ai::cloud::openrouter::DEFAULT_MODEL.to_string(),
            webhook_url: None,
            webhook_listen_addr: None,
            webhook_secret_token: None,
        }
    }
}

impl Config {
    /// Load configuration from environment variables, *.env* file and *cli*
    pub fn from_env() -> eyre::Result<Self> {
        let mut config = Config::default();

        // cli fetching
        let cli = Self::fetch_cli();

        let data_dir = dirs::data_local_dir().ok_or_eyre("data dir doesn't exist?")?;
        config.db_file = cli.db_file.unwrap_or(data_dir.join("tg-ai-admin/state.db"));

        config.webhook_url = cli.webhook_url;
        config.webhook_listen_addr = cli.webhook_listen_addr;

        // env
        let mut envs: Vec<HashMap<String, String, ahash::RandomState>> = Vec::default();

        envs.push(std::env::vars().collect());

        if let Some(Ok(env)) = cli.env_file.map(|f| dotenv::fetch_from_file(f)) {
            envs.push(env);
        }

        // 1 - std::env
        // 2 - .env file env
        //
        // 2 overwrites 1
        for env in envs {
            for (key, value) in env {
                match key.as_str() {
                    TELEGRAM_BOT_API_KEY_ENV => {
                        config.tg_bot_api_key = value;
                    }

                    OPENROUTER_API_KEY_ENV => {
                        config.openrouter_api_key = value;
                    }

                    TELEGRAM_WEBHOOK_SECRET_ENV => {
                        config.webhook_secret_token = Some(value);
                    }

                    _ => {
                        continue;
                    }
                }
            }
        }

        if config.is_tg_key_not_set() {
            return Err(eyre!("{TELEGRAM_BOT_API_KEY_ENV} env variable is not set"));
        } else if config.is_openrouter_not_set() {
            return Err(eyre!("{OPENROUTER_API_KEY_ENV} env variable is not set"));
        }

        Ok(config)
    }

    fn fetch_cli() -> Cli {
        Cli::parse()
    }
}

#[derive(Debug, Parser)]
#[command(
    about,
    version = formatcp!("{} / {}", crate::build::PKG_VERSION, crate::build::COMMIT_HASH),
    long_about = None,
)]
struct Cli {
    /// Path to .env-like file
    #[arg(long)]
    env_file: Option<PathBuf>,

    /// Path where database file will be located
    #[arg(long)]
    db_file: Option<PathBuf>,

    /// The model the bot will use
    #[arg(long, default_value_t = String::from(crate::ai::cloud::openrouter::DEFAULT_MODEL))]
    ai_model: String,

    /// Publically accessible URL for Telegram to send updates to (e.g. "https://example.org/tghook").
    /// If set, then the bot will start in the webhook mode.
    #[arg(long)]
    webhook_url: Option<Url>,

    /// Address and port to listen on for webhook updates (e.g. "0.0.0.0:8080")
    #[arg(long, default_value = "0.0.0.0:8080")]
    webhook_listen_addr: Option<SocketAddr>,
}
