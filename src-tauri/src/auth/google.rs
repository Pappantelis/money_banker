use crate::error::{AppError, Result};
use crate::models::CreateUser;
use chrono::{Duration, Utc};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use reqwest::Client as HttpClient;
use serde::Deserialize;

use super::tokens::OAuthTokens;

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v3/userinfo";

/// Google OAuth client with PKCE support
pub struct GoogleAuth {
    oauth_client: BasicClient,
    http_client: HttpClient,
}

/// Authorization request data (needed to complete the flow)
pub struct AuthorizationRequest {
    pub url: String,
    pub state: String,
    pub pkce_verifier: PkceCodeVerifier,
}

/// User info returned from Google
#[derive(Debug, Clone, Deserialize)]
pub struct GoogleUserInfo {
    pub sub: String, // Google ID
    pub email: String,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
}

impl GoogleAuth {
    /// Create new Google OAuth client
    pub fn new(client_id: &str, client_secret: &str, redirect_uri: &str) -> Result<Self> {
        if client_id.is_empty() || client_secret.is_empty() {
            return Err(AppError::Config(
                "Google OAuth credentials not configured. Set GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET".to_string(),
            ));
        }

        let auth_url = AuthUrl::new(GOOGLE_AUTH_URL.to_string())
            .map_err(|e| AppError::Config(format!("Invalid auth URL: {}", e)))?;

        let token_url = TokenUrl::new(GOOGLE_TOKEN_URL.to_string())
            .map_err(|e| AppError::Config(format!("Invalid token URL: {}", e)))?;

        let redirect_url = RedirectUrl::new(redirect_uri.to_string())
            .map_err(|e| AppError::Config(format!("Invalid redirect URI: {}", e)))?;

        let oauth_client = BasicClient::new(
            ClientId::new(client_id.to_string()),
            Some(ClientSecret::new(client_secret.to_string())),
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(redirect_url);

        let http_client = HttpClient::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| AppError::ExternalService(format!("HTTP client error: {}", e)))?;

        Ok(Self {
            oauth_client,
            http_client,
        })
    }

    /// Generate authorization URL with PKCE
    /// Returns the URL to open in browser and data needed to complete the flow
    pub fn create_authorization_request(&self) -> AuthorizationRequest {
        // Generate PKCE challenge (security best practice)
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate authorization URL
        let (auth_url, csrf_state) = self
            .oauth_client
            .authorize_url(CsrfToken::new_random)
            // Request offline access to get refresh token
            .add_extra_param("access_type", "offline")
            // Force consent screen to always get refresh token
            .add_extra_param("prompt", "consent")
            // Scopes
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/gmail.readonly".to_string(),
            ))
            .set_pkce_challenge(pkce_challenge)
            .url();

        AuthorizationRequest {
            url: auth_url.to_string(),
            state: csrf_state.secret().clone(),
            pkce_verifier,
        }
    }

    /// Exchange authorization code for tokens
    pub async fn exchange_code(
        &self,
        code: &str,
        pkce_verifier: PkceCodeVerifier,
    ) -> Result<OAuthTokens> {
        let token_result = self
            .oauth_client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await
            .map_err(|e| AppError::Auth(format!("Token exchange failed: {}", e)))?;

        // Calculate expiry time
        let expires_at = token_result
            .expires_in()
            .map(|duration| Utc::now() + Duration::seconds(duration.as_secs() as i64));

        Ok(OAuthTokens {
            access_token: token_result.access_token().secret().clone(),
            refresh_token: token_result.refresh_token().map(|t| t.secret().clone()),
            expires_at,
            token_type: "Bearer".to_string(),
        })
    }

    /// Refresh access token using refresh token
    pub async fn refresh_tokens(&self, refresh_token: &str) -> Result<OAuthTokens> {
        let token_result = self
            .oauth_client
            .exchange_refresh_token(&oauth2::RefreshToken::new(refresh_token.to_string()))
            .request_async(async_http_client)
            .await
            .map_err(|e| AppError::Auth(format!("Token refresh failed: {}", e)))?;

        let expires_at = token_result
            .expires_in()
            .map(|duration| Utc::now() + Duration::seconds(duration.as_secs() as i64));

        Ok(OAuthTokens {
            access_token: token_result.access_token().secret().clone(),
            // Refresh token may or may not be returned on refresh
            refresh_token: token_result
                .refresh_token()
                .map(|t| t.secret().clone())
                .or_else(|| Some(refresh_token.to_string())),
            expires_at,
            token_type: "Bearer".to_string(),
        })
    }

    /// Fetch user info from Google using access token
    pub async fn get_user_info(&self, access_token: &str) -> Result<GoogleUserInfo> {
        let response = self
            .http_client
            .get(GOOGLE_USERINFO_URL)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| AppError::ExternalService(format!("Failed to fetch user info: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Auth(format!(
                "Google API error {}: {}",
                status, body
            )));
        }

        response
            .json::<GoogleUserInfo>()
            .await
            .map_err(|e| AppError::ExternalService(format!("Failed to parse user info: {}", e)))
    }
}

/// Convert Google user info to CreateUser for database
impl From<GoogleUserInfo> for CreateUser {
    fn from(info: GoogleUserInfo) -> Self {
        CreateUser {
            google_id: info.sub,
            email: info.email,
            f_name: info.given_name.unwrap_or_else(|| "Unknown".to_string()),
            l_name: info.family_name.unwrap_or_default(),
            photo_url: info.picture,
        }
    }
}
