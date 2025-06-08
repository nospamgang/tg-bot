use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Request, State},
    middleware::{Next, from_fn_with_state},
    response::{IntoResponse, Response},
    routing::post,
};
use eyre::Context;
use frankenstein::updates::Update;
use reqwest::StatusCode;
use tokio::net::TcpListener;
use tracing::{error, info, warn};

use crate::service::service::Service;

pub struct WebhookListener {
    listen_addr: String,
    service: Arc<Service>,
    secret_token: Option<String>,
    webhook_path: String,
}

struct SharedState {
    service: Arc<Service>,
    secret_token: Option<String>,
}

impl WebhookListener {
    pub fn new(
        listen_addr: String,
        webhook_path: String,
        service: Arc<Service>,
        secret_token: Option<String>,
    ) -> Self {
        Self {
            listen_addr,
            service,
            secret_token,
            webhook_path,
        }
    }

    pub async fn listen(self) -> eyre::Result<()> {
        info!("Initializing webhook server...");

        let state = Arc::new(SharedState {
            service: self.service,
            secret_token: self.secret_token,
        });

        info!("Starting listener on path: {}", self.webhook_path);

        let app = Router::new()
            .route(
                &self.webhook_path,
                post(Self::webhook_handler).route_layer(from_fn_with_state(
                    state.clone(),
                    Self::secret_token_middleware,
                )),
            )
            .with_state(state);

        let listener = TcpListener::bind(&self.listen_addr)
            .await
            .wrap_err("failed to bind address")?;

        info!("Webhook server listening on {}", listener.local_addr()?);

        axum::serve(listener, app)
            .await
            .wrap_err("webhook server failed")?;

        Ok(())
    }

    /// Axum middleware to verify the `X-Telegram-Bot-Api-Secret-Token` header
    async fn secret_token_middleware(
        State(state): State<Arc<SharedState>>,
        request: Request,
        next: Next,
    ) -> Response {
        let configured_token = match state.secret_token.as_ref() {
            Some(token) => token,

            // no token configured on our end, so we don't need to check for one
            None => return next.run(request).await,
        };

        let provided_token = request
            .headers()
            .get("X-Telegram-Bot-Api-Secret-Token")
            .and_then(|v| v.to_str().ok());

        if provided_token.is_some_and(|v| v == configured_token) {
            next.run(request).await
        } else {
            warn!("Rejected webhook request due to missing/invalid secret token");

            StatusCode::UNAUTHORIZED.into_response()
        }
    }

    async fn webhook_handler(State(state): State<Arc<SharedState>>, Json(update): Json<Update>) {
        let service = state.service.clone();

        tokio::spawn(async move {
            if let Err(e) = service.dispatch_update(update).await {
                error!("Error handling update from webhook: {e:?}");
            }
        });
    }
}
