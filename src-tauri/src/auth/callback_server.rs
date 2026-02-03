use crate::error::{AppError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use warp::Filter;

const SUCCESS_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Login Successful</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }
        .container {
            text-align: center;
            padding: 40px;
            background: rgba(255,255,255,0.1);
            border-radius: 20px;
            backdrop-filter: blur(10px);
        }
        h1 { margin-bottom: 10px; }
        p { opacity: 0.9; }
    </style>
</head>
<body>
    <div class="container">
        <h1>✓ Login Successful!</h1>
        <p>You can close this window and return to the app.</p>
    </div>
</body>
</html>
"#;

const ERROR_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Login Failed</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
            background: #ff6b6b;
            color: white;
        }
        .container {
            text-align: center;
            padding: 40px;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>✗ Login Failed</h1>
        <p>Invalid state parameter. Please try again.</p>
    </div>
</body>
</html>
"#;

/// OAuth callback result
#[derive(Debug, Clone)]
pub struct CallbackResult {
    pub code: String,
}

/// Local HTTP server that catches the OAuth callback
pub struct CallbackServer {
    port: u16,
}

impl CallbackServer {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    /// Wait for OAuth callback from Google
    /// Returns the authorization code when received
    pub async fn wait_for_callback(&self, expected_state: String) -> Result<String> {
        let (tx, mut rx) = mpsc::channel::<String>(1);
        let tx = Arc::new(tx);
        let expected = Arc::new(expected_state);

        // Create the callback route
        let tx_clone = tx.clone();
        let expected_clone = expected.clone();

        let callback = warp::path("callback")
            .and(warp::query::<HashMap<String, String>>())
            .map(move |params: HashMap<String, String>| {
                let code = params.get("code").cloned().unwrap_or_default();
                let state = params.get("state").cloned().unwrap_or_default();

                if state == *expected_clone {
                    // Send the code through the channel
                    let _ = tx_clone.try_send(code);
                    warp::reply::html(SUCCESS_HTML)
                } else {
                    warp::reply::html(ERROR_HTML)
                }
            });

        // Bind the server
        let addr = ([127, 0, 0, 1], self.port);

        tracing::info!("OAuth callback server listening on http://localhost:{}/callback", self.port);

        // Spawn the server
        let (_, server) =
            warp::serve(callback).bind_with_graceful_shutdown(addr, async move {
                // Keep server alive for 5 minutes max
                tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
            });

        let server_handle = tokio::spawn(server);

        // Wait for callback or timeout
        let result = tokio::time::timeout(tokio::time::Duration::from_secs(300), rx.recv()).await;

        // Abort the server
        server_handle.abort();

        match result {
            Ok(Some(code)) => {
                tracing::info!("OAuth callback received successfully");
                Ok(code)
            }
            Ok(None) => Err(AppError::Auth("OAuth callback channel closed".to_string())),
            Err(_) => Err(AppError::Auth(
                "OAuth callback timeout (5 minutes)".to_string(),
            )),
        }
    }
}
