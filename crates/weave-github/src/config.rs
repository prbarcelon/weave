use std::sync::Arc;

/// Server configuration loaded from environment variables.
#[derive(Clone)]
pub struct Config {
    /// GitHub App ID.
    pub app_id: u64,
    /// GitHub App private key (PEM format).
    pub private_key: String,
    /// Webhook secret for HMAC-SHA256 verification.
    pub webhook_secret: String,
    /// Port to listen on.
    pub port: u16,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// Required: GITHUB_APP_ID, GITHUB_PRIVATE_KEY, GITHUB_WEBHOOK_SECRET
    /// Optional: PORT (default 8080)
    pub fn from_env() -> Result<Arc<Self>, String> {
        let app_id: u64 = std::env::var("GITHUB_APP_ID")
            .map_err(|_| "GITHUB_APP_ID not set")?
            .parse()
            .map_err(|_| "GITHUB_APP_ID must be a number")?;

        let private_key =
            std::env::var("GITHUB_PRIVATE_KEY").map_err(|_| "GITHUB_PRIVATE_KEY not set")?;

        let webhook_secret =
            std::env::var("GITHUB_WEBHOOK_SECRET").map_err(|_| "GITHUB_WEBHOOK_SECRET not set")?;

        let port: u16 = std::env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .map_err(|_| "PORT must be a number")?;

        Ok(Arc::new(Config {
            app_id,
            private_key,
            webhook_secret,
            port,
        }))
    }
}
