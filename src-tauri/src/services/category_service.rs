use crate::db::repository::CategoryRepository;
use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models::{Category, CreateCategory};
use uuid::Uuid;

pub struct CategoryService {
    pool: DbPool,
}

impl CategoryService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create_category(&self, category: CreateCategory) -> Result<Category> {
        if category.name.trim().is_empty() {
            return Err(AppError::Validation(
                "Category name cannot be empty".to_string(),
            ));
        }

        CategoryRepository::create(&self.pool, category).await
    }

    pub async fn get_category(&self, id: Uuid) -> Result<Option<Category>> {
        CategoryRepository::find_by_id(&self.pool, id).await
    }

    pub async fn get_user_categories(&self, user_id: Uuid) -> Result<Vec<Category>> {
        CategoryRepository::find_by_user(&self.pool, user_id).await
    }

    pub async fn get_income_categories(&self, user_id: Uuid) -> Result<Vec<Category>> {
        CategoryRepository::find_income_categories(&self.pool, user_id).await
    }

    pub async fn get_expense_categories(&self, user_id: Uuid) -> Result<Vec<Category>> {
        CategoryRepository::find_expense_categories(&self.pool, user_id).await
    }

    pub async fn delete_category(&self, id: Uuid) -> Result<()> {
        CategoryRepository::delete(&self.pool, id).await
    }

    /// Create default categories for a new user
    pub async fn create_default_categories(&self, user_id: Uuid) -> Result<Vec<Category>> {
        let defaults = vec![
            ("Supermarket", Some("cart"), false),
            ("Fuel", Some("gas-pump"), false),
            ("Entertainment", Some("film"), false),
            ("Bills", Some("file-invoice"), false),
            ("Dining", Some("utensils"), false),
            ("Shopping", Some("bag"), false),
            ("Healthcare", Some("heart"), false),
            ("Transport", Some("bus"), false),
            ("Other", None, false),
            ("Salary", Some("briefcase"), true),
            ("Other Income", Some("plus"), true),
        ];

        let mut categories = Vec::new();
        for (name, icon, is_income) in defaults {
            let cat = CategoryRepository::create(
                &self.pool,
                CreateCategory {
                    user_id,
                    name: name.to_string(),
                    icon: icon.map(|s| s.to_string()),
                    is_income,
                },
            )
            .await?;
            categories.push(cat);
        }

        Ok(categories)
    }
}
