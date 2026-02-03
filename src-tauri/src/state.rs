use crate::{models::User, DbPool};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Application state managed by Tauri
pub struct AppState {
    pub pool: DbPool,
    pub current_user: Arc<RwLock<Option<User>>>,
}

impl AppState {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool,
            current_user: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn set_user(&self, user: Option<User>) {
        let mut guard = self.current_user.write().await;
        *guard = user;
    }

    pub async fn get_user(&self) -> Option<User> {
        let guard = self.current_user.read().await;
        guard.clone()
    }
}
