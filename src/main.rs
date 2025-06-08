mod ai;
mod config;
mod dotenv;
mod service;
mod telegram;

use std::sync::Arc;

use eyre::{Context, OptionExt};
use fjall::Config as FjallConfig;
use frankenstein::types::AllowedUpdate;
use reqwest::Client as ReqwestClient;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt as _};

use crate::{
    ai::cloud::openrouter::Openrouter,
    config::Config,
    service::{service::Service, webhook::WebhookListener},
};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    info!("starting tg-ai-moderator...");

    info!(
        "Version: {}, Commit: {}",
        crate::build::PKG_VERSION,
        crate::build::SHORT_COMMIT
    );

    let config = Arc::new(Config::from_env()?);
    let http_client = Arc::new(ReqwestClient::new());

    // init DB
    let db_path = config.db_file();
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).wrap_err("failed to create DB directory")?;
    }
    let db = Arc::new(FjallConfig::new(db_path).open()?);
    info!("database opened at: {}", db_path.display());

    // init AI provider
    let ai_provider = Arc::new(Openrouter::new(
        http_client.clone(),
        Arc::new(config.openrouter_api_key().to_string()),
        Arc::new(crate::ai::cloud::openrouter::DEFAULT_MODEL.to_string()),
    ));

    // init service
    let service = Arc::new(Service::new(config.clone(), http_client, ai_provider, db));

    if let Err(e) = service.update_cas_banned_ids().await {
        error!("failed to perform initial CAS list fetch: {e:?}");
    }

    if config.is_webhook_active() {
        info!("starting in WEBHOOK mode");

        let webhook_url = config
            .webhook_url()
            .ok_or_eyre("webhook URL must be set for webhook mode")?;
        let listen_addr = config
            .webhook_listen_addr()
            .ok_or_eyre("webhook listen address must be set for webhook mode")?;
        let secret_token = config.webhook_secret_token().map(|s| s.to_string());

        info!("setting webhook to: {webhook_url} via API");
        service
            .bot()
            .set_webhook(
                vec![AllowedUpdate::Message],
                webhook_url.to_string(),
                secret_token.clone(),
            )
            .await?;
        info!("webhook set successfully");

        service.spawn_background_tasks();

        let listener = WebhookListener::new(
            listen_addr.to_string(),
            webhook_url.path().to_string(),
            service,
            secret_token,
        );

        listener.listen().await?;
    } else {
        info!("starting in POLLING mode");

        service.start_polling().await?;
    }

    Ok(())
}

mod build {
    pub use shadow_build::*;

    shadow_rs::shadow!(shadow_build);
}
