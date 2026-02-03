use crate::error::Result;
use crate::models::{CreateTransaction, Transaction, TransactionFilter};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

pub struct TransactionRepository;

#[derive(Debug)]
pub struct MonthlySummary {
    pub income: Decimal,
    pub expenses: Decimal,
    pub transaction_count: i64,
}

impl TransactionRepository {
    pub async fn create(pool: &PgPool, tx: CreateTransaction) -> Result<Transaction> {
        let transaction = sqlx::query_as::<_, Transaction>(
            r#"
            INSERT INTO transactions
                (user_id, category_id, amount, store, description, source, email_message_id, transaction_date)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(tx.user_id)
        .bind(tx.category_id)
        .bind(tx.amount)
        .bind(&tx.store)
        .bind(&tx.description)
        .bind(&tx.source)
        .bind(&tx.email_message_id)
        .bind(tx.transaction_date)
        .fetch_one(pool)
        .await?;

        Ok(transaction)
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Transaction>> {
        let tx = sqlx::query_as::<_, Transaction>("SELECT * FROM transactions WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        Ok(tx)
    }

    pub async fn find_by_user(
        pool: &PgPool,
        user_id: Uuid,
        filter: &TransactionFilter,
    ) -> Result<Vec<Transaction>> {
        let limit = filter.limit.unwrap_or(100) as i64;
        let offset = filter.offset.unwrap_or(0) as i64;

        let transactions = sqlx::query_as::<_, Transaction>(
            r#"
            SELECT * FROM transactions
            WHERE user_id = $1
              AND ($2::date IS NULL OR transaction_date >= $2)
              AND ($3::date IS NULL OR transaction_date <= $3)
              AND ($4::uuid IS NULL OR category_id = $4)
            ORDER BY transaction_date DESC, created_at DESC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(user_id)
        .bind(filter.start_date)
        .bind(filter.end_date)
        .bind(filter.category_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(transactions)
    }

    pub async fn get_monthly_summary(
        pool: &PgPool,
        user_id: Uuid,
        year: i32,
        month: u32,
    ) -> Result<MonthlySummary> {
        let row = sqlx::query_as::<_, (Decimal, Decimal, i64)>(
            r#"
            SELECT
                COALESCE(SUM(CASE WHEN c.is_income THEN t.amount ELSE 0 END), 0) as income,
                COALESCE(SUM(CASE WHEN NOT c.is_income OR c.is_income IS NULL THEN ABS(t.amount) ELSE 0 END), 0) as expenses,
                COUNT(*) as transaction_count
            FROM transactions t
            LEFT JOIN categories c ON t.category_id = c.id
            WHERE t.user_id = $1
              AND EXTRACT(YEAR FROM t.transaction_date) = $2
              AND EXTRACT(MONTH FROM t.transaction_date) = $3
            "#,
        )
        .bind(user_id)
        .bind(year)
        .bind(month as i32)
        .fetch_one(pool)
        .await?;

        Ok(MonthlySummary {
            income: row.0,
            expenses: row.1,
            transaction_count: row.2,
        })
    }

    pub async fn find_by_email_message_id(
        pool: &PgPool,
        email_message_id: &str,
    ) -> Result<Option<Transaction>> {
        let tx = sqlx::query_as::<_, Transaction>(
            "SELECT * FROM transactions WHERE email_message_id = $1",
        )
        .bind(email_message_id)
        .fetch_optional(pool)
        .await?;

        Ok(tx)
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        category_id: Option<Uuid>,
        amount: Option<Decimal>,
        store: Option<String>,
        description: Option<String>,
    ) -> Result<Transaction> {
        let tx = sqlx::query_as::<_, Transaction>(
            r#"
            UPDATE transactions
            SET category_id = COALESCE($2, category_id),
                amount = COALESCE($3, amount),
                store = COALESCE($4, store),
                description = COALESCE($5, description)
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(category_id)
        .bind(amount)
        .bind(store)
        .bind(description)
        .fetch_one(pool)
        .await?;

        Ok(tx)
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM transactions WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }
}
