use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// OAuth tokens received from Google
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub token_type: String,
}

impl OAuthTokens {
    /// Check if access token is expired (with 5 minute buffer)
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires) => Utc::now() >= expires - Duration::minutes(5),
            None => false,
        }
    }

    /// Check if tokens need refresh
    pub fn needs_refresh(&self) -> bool {
        self.is_expired() && self.refresh_token.is_some()
    }
}

/// Stored authentication data (persisted in keyring)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredAuth {
    pub tokens: OAuthTokens,
    pub user_id: String,
    pub google_id: String,
    pub email: String,
}
