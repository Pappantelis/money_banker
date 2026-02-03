use crate::db::repository::{MonthlySummary, TransactionRepository};
use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models::{CreateTransaction, Transaction, TransactionFilter};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

pub struct TransactionService {
    pool: DbPool,
}

impl TransactionService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create_transaction(
        &self,
        user_id: Uuid,
        amount: Decimal,
        store: Option<String>,
        description: Option<String>,
        category_id: Option<Uuid>,
        transaction_date: NaiveDate,
        source: String,
    ) -> Result<Transaction> {
        if amount == Decimal::ZERO {
            return Err(AppError::Validation("Amount cannot be zero".to_string()));
        }

        let tx = CreateTransaction {
            user_id,
            amount,
            store,
            description,
            category_id,
            transaction_date,
            source,
            email_message_id: None,
        };

        TransactionRepository::create(&self.pool, tx).await
    }

    pub async fn create_transaction_from_dto(&self, tx: CreateTransaction) -> Result<Transaction> {
        if tx.amount == Decimal::ZERO {
            return Err(AppError::Validation("Amount cannot be zero".to_string()));
        }

        TransactionRepository::create(&self.pool, tx).await
    }

    pub async fn get_transaction(&self, id: Uuid) -> Result<Option<Transaction>> {
        TransactionRepository::find_by_id(&self.pool, id).await
    }

    pub async fn get_transactions(
        &self,
        user_id: Uuid,
        filter: TransactionFilter,
    ) -> Result<Vec<Transaction>> {
        TransactionRepository::find_by_user(&self.pool, user_id, &filter).await
    }

    /// Get all transactions for a user with a limit
    pub async fn get_user_transactions(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<Transaction>> {
        let filter = TransactionFilter {
            limit: Some(limit),
            ..Default::default()
        };
        TransactionRepository::find_by_user(&self.pool, user_id, &filter).await
    }

    /// Get transactions for a specific month
    pub async fn get_user_transactions_by_month(
        &self,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> Result<Vec<Transaction>> {
        // Calculate start and end dates for the month
        let start_date = NaiveDate::from_ymd_opt(year, month, 1)
            .ok_or_else(|| AppError::Validation("Invalid year/month".to_string()))?;

        // Get the last day of the month
        let end_date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        }
        .map(|d| d.pred_opt().unwrap_or(d))
        .ok_or_else(|| AppError::Validation("Invalid year/month".to_string()))?;

        let filter = TransactionFilter {
            start_date: Some(start_date),
            end_date: Some(end_date),
            ..Default::default()
        };
        TransactionRepository::find_by_user(&self.pool, user_id, &filter).await
    }

    pub async fn get_monthly_summary(
        &self,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> Result<MonthlySummary> {
        TransactionRepository::get_monthly_summary(&self.pool, user_id, year, month).await
    }

    /// Check if an email message has already been imported
    pub async fn is_email_imported(&self, email_message_id: &str) -> Result<bool> {
        let tx =
            TransactionRepository::find_by_email_message_id(&self.pool, email_message_id).await?;
        Ok(tx.is_some())
    }

    pub async fn update_transaction(
        &self,
        id: Uuid,
        category_id: Option<Uuid>,
        amount: Option<Decimal>,
        store: Option<String>,
        description: Option<String>,
    ) -> Result<Transaction> {
        TransactionRepository::update(&self.pool, id, category_id, amount, store, description).await
    }

    pub async fn delete_transaction(&self, id: Uuid) -> Result<()> {
        TransactionRepository::delete(&self.pool, id).await
    }
}
