//! Authentication module for Google OAuth
//!
//! Provides:
//! - `perform_google_login()` - Full OAuth flow with browser
//! - `try_restore_session()` - Auto-login from stored tokens

mod callback_server;
mod google;
mod token_storage;
mod tokens;

pub use google::{GoogleAuth, GoogleUserInfo};
pub use token_storage::TokenStorage;
pub use tokens::{OAuthTokens, StoredAuth};

use crate::config::Settings;
use crate::db::repository::UserRepository;
use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models::User;
use callback_server::CallbackServer;

/// Perform full Google OAuth login flow
///
/// 1. Starts local callback server
/// 2. Opens browser for Google login
/// 3. Waits for callback with auth code
/// 4. Exchanges code for tokens
/// 5. Fetches user info
/// 6. Creates/updates user in database
/// 7. Stores tokens in system keyring
pub async fn perform_google_login(settings: &Settings, pool: &DbPool) -> Result<User> {
    // Create OAuth client
    let google_auth = GoogleAuth::new(
        &settings.google_client_id,
        &settings.google_client_secret,
        &settings.oauth_redirect_uri(),
    )?;

    // Generate authorization URL with PKCE
    let auth_request = google_auth.create_authorization_request();

    // Create callback server
    let callback_server = CallbackServer::new(settings.oauth_callback_port);

    // Open browser for user login
    tracing::info!("Opening browser for Google login...");
    println!("\n🔐 Opening browser for Google login...");
    println!("   If the browser doesn't open, visit:");
    println!("   {}\n", auth_request.url);

    open::that(&auth_request.url).map_err(|e| {
        AppError::ExternalService(format!(
            "Failed to open browser: {}. Please open the URL manually.",
            e
        ))
    })?;

    // Wait for callback
    tracing::info!("Waiting for OAuth callback...");
    println!("⏳ Waiting for authentication...");

    let auth_code = callback_server
        .wait_for_callback(auth_request.state)
        .await?;

    // Exchange code for tokens
    tracing::info!("Exchanging authorization code for tokens...");
    let tokens = google_auth
        .exchange_code(&auth_code, auth_request.pkce_verifier)
        .await?;

    // Get user info from Google
    let user_info = google_auth.get_user_info(&tokens.access_token).await?;
    tracing::info!("Logged in as: {} ({})", user_info.email, user_info.sub);

    // Find or create user in database
    let user = UserRepository::find_or_create_by_google(pool, user_info.clone().into()).await?;

    // Store tokens securely
    let storage = TokenStorage::new()?;
    let stored_auth = StoredAuth {
        tokens,
        user_id: user.id.to_string(),
        google_id: user_info.sub,
        email: user_info.email.clone(),
    };
    storage.store(&stored_auth)?;

    println!("✅ Login successful! Welcome, {}!", user.full_name());
    tracing::info!("Authentication complete for user: {}", user.email);

    Ok(user)
}

/// Try to restore session from stored tokens
///
/// Returns `Ok(Some(user))` if session restored successfully
/// Returns `Ok(None)` if no stored tokens or tokens invalid
pub async fn try_restore_session(settings: &Settings, pool: &DbPool) -> Result<Option<User>> {
    let storage = TokenStorage::new()?;

    // Check for stored auth
    let stored_auth = match storage.load()? {
        Some(auth) => auth,
        None => {
            tracing::debug!("No stored authentication found");
            return Ok(None);
        }
    };

    tracing::info!("Found stored session for {}", stored_auth.email);

    // Create OAuth client for potential token refresh
    let google_auth = GoogleAuth::new(
        &settings.google_client_id,
        &settings.google_client_secret,
        &settings.oauth_redirect_uri(),
    )?;

    // Check if tokens need refresh
    let tokens = if stored_auth.tokens.needs_refresh() {
        match &stored_auth.tokens.refresh_token {
            Some(refresh_token) => {
                tracing::info!("Access token expired, refreshing...");
                match google_auth.refresh_tokens(refresh_token).await {
                    Ok(new_tokens) => {
                        // Update stored tokens
                        let updated_auth = StoredAuth {
                            tokens: new_tokens.clone(),
                            ..stored_auth.clone()
                        };
                        storage.store(&updated_auth)?;
                        tracing::info!("Tokens refreshed successfully");
                        new_tokens
                    }
                    Err(e) => {
                        tracing::warn!("Token refresh failed: {}, clearing session", e);
                        storage.clear()?;
                        return Ok(None);
                    }
                }
            }
            None => {
                tracing::warn!("Token expired with no refresh token, clearing session");
                storage.clear()?;
                return Ok(None);
            }
        }
    } else {
        stored_auth.tokens.clone()
    };

    // Verify tokens still work by fetching user info
    match google_auth.get_user_info(&tokens.access_token).await {
        Ok(info) => {
            tracing::debug!("Token verified for {}", info.email);
        }
        Err(e) => {
            tracing::warn!("Token validation failed: {}, clearing session", e);
            storage.clear()?;
            return Ok(None);
        }
    }

    // Load user from database
    let user = UserRepository::find_by_google_id(pool, &stored_auth.google_id).await?;

    match user {
        Some(u) => {
            tracing::info!("Session restored for {}", u.email);
            Ok(Some(u))
        }
        None => {
            tracing::warn!("User not found in database, clearing session");
            storage.clear()?;
            Ok(None)
        }
    }
}

/// Logout - clear stored tokens
pub async fn logout() -> Result<()> {
    let storage = TokenStorage::new()?;
    storage.clear()?;
    tracing::info!("User logged out");
    println!("👋 Logged out successfully!");
    Ok(())
}

/// Check if user is logged in (has stored tokens)
pub fn is_logged_in() -> bool {
    TokenStorage::new()
        .map(|s| s.has_stored_auth())
        .unwrap_or(false)
}

/// Get stored access token (for API calls)
/// Returns None if not logged in or token expired
pub async fn get_access_token(settings: &Settings) -> Result<Option<String>> {
    let storage = TokenStorage::new()?;

    let stored_auth = match storage.load()? {
        Some(auth) => auth,
        None => return Ok(None),
    };

    // If token is expired, try to refresh
    if stored_auth.tokens.needs_refresh() {
        if let Some(refresh_token) = &stored_auth.tokens.refresh_token {
            let google_auth = GoogleAuth::new(
                &settings.google_client_id,
                &settings.google_client_secret,
                &settings.oauth_redirect_uri(),
            )?;

            let new_tokens = google_auth.refresh_tokens(refresh_token).await?;

            let updated_auth = StoredAuth {
                tokens: new_tokens.clone(),
                ..stored_auth
            };
            storage.store(&updated_auth)?;

            return Ok(Some(new_tokens.access_token));
        } else {
            return Ok(None);
        }
    }

    Ok(Some(stored_auth.tokens.access_token))
}
