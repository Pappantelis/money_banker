use crate::error::{AppError, Result};

use super::tokens::StoredAuth;

const SERVICE_NAME: &str = "montlhy_bank_usage";
const TOKEN_KEY: &str = "google_oauth_tokens";

/// Secure token storage using system keyring
/// - macOS: Keychain
/// - Windows: Credential Manager
/// - Linux: Secret Service (GNOME Keyring, KWallet)
pub struct TokenStorage {
    entry: keyring::Entry,
}

impl TokenStorage {
    /// Create new token storage instance
    pub fn new() -> Result<Self> {
        let entry = keyring::Entry::new(SERVICE_NAME, TOKEN_KEY)
            .map_err(|e| AppError::Auth(format!("Failed to initialize keyring: {}", e)))?;
        Ok(Self { entry })
    }

    /// Store authentication data securely
    pub fn store(&self, auth: &StoredAuth) -> Result<()> {
        let json = serde_json::to_string(auth)
            .map_err(|e| AppError::Auth(format!("Failed to serialize tokens: {}", e)))?;

        self.entry
            .set_password(&json)
            .map_err(|e| AppError::Auth(format!("Failed to store tokens in keyring: {}", e)))?;

        tracing::debug!("Tokens stored securely in system keyring");
        Ok(())
    }

    /// Load stored authentication data
    pub fn load(&self) -> Result<Option<StoredAuth>> {
        match self.entry.get_password() {
            Ok(json) => {
                let auth: StoredAuth = serde_json::from_str(&json)
                    .map_err(|e| AppError::Auth(format!("Failed to deserialize tokens: {}", e)))?;
                Ok(Some(auth))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::Auth(format!(
                "Failed to load tokens from keyring: {}",
                e
            ))),
        }
    }

    /// Clear stored authentication data (logout)
    pub fn clear(&self) -> Result<()> {
        match self.entry.delete_credential() {
            Ok(_) => {
                tracing::debug!("Tokens cleared from keyring");
                Ok(())
            }
            Err(keyring::Error::NoEntry) => Ok(()), // Already cleared
            Err(e) => Err(AppError::Auth(format!(
                "Failed to clear tokens from keyring: {}",
                e
            ))),
        }
    }

    /// Check if tokens exist
    pub fn has_stored_auth(&self) -> bool {
        self.entry.get_password().is_ok()
    }
}
