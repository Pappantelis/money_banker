pub mod auth;
pub mod commands;
pub mod config;
pub mod db;
pub mod error;
pub mod models;
pub mod services;
pub mod state;

// Re-exports for convenience
pub use config::Settings;
pub use db::DbPool;
pub use error::{AppError, Result};

use commands::*;
use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Initialize in a blocking task
            let handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                initialize_app(handle).await
            })?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_current_user,
            login,
            logout,
            get_categories,
            get_transactions,
            add_transaction,
            get_monthly_summary,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn initialize_app(handle: tauri::AppHandle) -> std::result::Result<(), Box<dyn std::error::Error>> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    // Initialize tracing/logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let settings = Settings::load()?;

    tracing::info!("Starting Monthly Bank Usage application...");

    // Initialize database pool
    let pool = db::create_pool(&settings.database_url).await?;
    tracing::info!("Connected to database");

    // Run migrations
    db::run_migrations(&pool).await?;

    // Create app state
    let state = AppState::new(pool.clone());

    // Try to restore existing session
    if let Ok(Some(user)) = auth::try_restore_session(&settings, &pool).await {
        tracing::info!("Session restored for user: {}", user.email);
        state.set_user(Some(user)).await;
    }

    // Manage state with Tauri
    handle.manage(state);

    Ok(())
}
