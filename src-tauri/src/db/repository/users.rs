use crate::error::Result;
use crate::models::{CreateUser, UpdateUser, User};
use sqlx::PgPool;
use uuid::Uuid;

pub struct UserRepository;

impl UserRepository {
    pub async fn create(pool: &PgPool, user: CreateUser) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (google_id, email, f_name, l_name, photo_url)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(&user.google_id)
        .bind(&user.email)
        .bind(&user.f_name)
        .bind(&user.l_name)
        .bind(&user.photo_url)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        Ok(user)
    }

    pub async fn find_by_google_id(pool: &PgPool, google_id: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE google_id = $1")
            .bind(google_id)
            .fetch_optional(pool)
            .await?;

        Ok(user)
    }

    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await?;

        Ok(user)
    }

    pub async fn find_or_create_by_google(pool: &PgPool, user: CreateUser) -> Result<User> {
        // Try to find existing user first
        if let Some(existing) = Self::find_by_google_id(pool, &user.google_id).await? {
            return Ok(existing);
        }

        // Create new user
        Self::create(pool, user).await
    }

    pub async fn update(pool: &PgPool, id: Uuid, user: UpdateUser) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET f_name = COALESCE($2, f_name),
                l_name = COALESCE($3, l_name),
                photo_url = COALESCE($4, photo_url)
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&user.f_name)
        .bind(&user.l_name)
        .bind(&user.photo_url)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }
}
