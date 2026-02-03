use crate::db::repository::UserRepository;
use crate::db::DbPool;
use crate::error::Result;
use crate::models::{CreateUser, User};
use uuid::Uuid;

pub struct UserService {
    pool: DbPool,
}

impl UserService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn get_user(&self, id: Uuid) -> Result<Option<User>> {
        UserRepository::find_by_id(&self.pool, id).await
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        UserRepository::find_by_email(&self.pool, email).await
    }

    pub async fn authenticate_google(&self, google_user: CreateUser) -> Result<User> {
        UserRepository::find_or_create_by_google(&self.pool, google_user).await
    }
}
