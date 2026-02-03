use crate::error::Result;
use crate::models::{Category, CreateCategory};
use sqlx::PgPool;
use uuid::Uuid;

pub struct CategoryRepository;

impl CategoryRepository {
    pub async fn create(pool: &PgPool, category: CreateCategory) -> Result<Category> {
        let category = sqlx::query_as::<_, Category>(
            r#"
            INSERT INTO categories (user_id, name, icon, is_income)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(category.user_id)
        .bind(&category.name)
        .bind(&category.icon)
        .bind(category.is_income)
        .fetch_one(pool)
        .await?;

        Ok(category)
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Category>> {
        let category = sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        Ok(category)
    }

    pub async fn find_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<Category>> {
        let categories = sqlx::query_as::<_, Category>(
            "SELECT * FROM categories WHERE user_id = $1 ORDER BY name",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(categories)
    }

    pub async fn find_income_categories(pool: &PgPool, user_id: Uuid) -> Result<Vec<Category>> {
        let categories = sqlx::query_as::<_, Category>(
            "SELECT * FROM categories WHERE user_id = $1 AND is_income = true ORDER BY name",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(categories)
    }

    pub async fn find_expense_categories(pool: &PgPool, user_id: Uuid) -> Result<Vec<Category>> {
        let categories = sqlx::query_as::<_, Category>(
            "SELECT * FROM categories WHERE user_id = $1 AND is_income = false ORDER BY name",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(categories)
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM categories WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }
}
