use std::{collections::HashSet, sync::Arc};

use const_format::formatcp;
use eyre::Context;
use futures::StreamExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_util::io::StreamReader;
use tracing::warn;

use crate::telegram::bot::UserId;

pub struct Client {
    http_client: Arc<reqwest::Client>,
}

impl Client {
    pub const BASE_URL: &str = "https://api.cas.chat";

    pub fn new(http_client: Arc<reqwest::Client>) -> Self {
        Self { http_client }
    }

    // CAS API always returns `false` for some reason for any ID, so it won't be implemented
    // pub async fn check_user(&self) -> eyre::Result<()> { }

    /// Fetch all banned accounts' IDs banned by Combot Anti-Spam
    pub async fn fetch_full_list(&self) -> eyre::Result<HashSet<UserId, ahash::RandomState>> {
        let resp = self
            .http_client
            .get(formatcp!("{}/{}", Client::BASE_URL, "export.csv"))
            .send()
            .await
            .wrap_err("unable to complete the request")?;

        let mut output = HashSet::default();

        let resp_reader =
            BufReader::new(StreamReader::new(resp.bytes_stream().map(|result| {
                result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
            })));

        let mut lines = resp_reader.lines();

        while let Some(line) = lines.next_line().await? {
            if line.is_empty() {
                continue;
            }

            match line.parse::<UserId>() {
                Ok(id) => {
                    output.insert(id);
                }
                Err(e) => {
                    warn!("failed to parse user ID from CAS export: '{line}': {e}");
                }
            }
        }

        Ok(output)
    }
}
