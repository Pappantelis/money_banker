use crate::error::{AppError, Result};

const DEFAULT_OAUTH_CALLBACK_PORT: u16 = 8085;

#[derive(Debug, Clone)]
pub struct Settings {
    pub database_url: String,
    pub log_level: String,
    // Google OAuth settings
    pub google_client_id: String,
    pub google_client_secret: String,
    pub oauth_callback_port: u16,
}

impl Settings {
    pub fn load() -> Result<Self> {
        dotenvy::dotenv().ok();

        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| AppError::Config("DATABASE_URL must be set".to_string()))?;

        let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        // Google OAuth - required for authentication
        let google_client_id = std::env::var("GOOGLE_CLIENT_ID")
            .map_err(|_| AppError::Config("GOOGLE_CLIENT_ID must be set".to_string()))?;

        let google_client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
            .map_err(|_| AppError::Config("GOOGLE_CLIENT_SECRET must be set".to_string()))?;

        let oauth_callback_port = std::env::var("OAUTH_CALLBACK_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_OAUTH_CALLBACK_PORT);

        Ok(Self {
            database_url,
            log_level,
            google_client_id,
            google_client_secret,
            oauth_callback_port,
        })
    }

    /// Get the OAuth redirect URI for Google
    pub fn oauth_redirect_uri(&self) -> String {
        format!("http://localhost:{}/callback", self.oauth_callback_port)
    }
}
