use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub category_id: Option<Uuid>,
    pub amount: Decimal,
    pub store: Option<String>,
    pub description: Option<String>,
    pub source: Option<String>,
    pub email_message_id: Option<String>,
    pub transaction_date: NaiveDate,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTransaction {
    pub user_id: Uuid,
    pub category_id: Option<Uuid>,
    pub amount: Decimal,
    pub store: Option<String>,
    pub description: Option<String>,
    #[serde(default = "default_source")]
    pub source: String,
    pub email_message_id: Option<String>,
    pub transaction_date: NaiveDate,
}

fn default_source() -> String {
    "manual".to_string()
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TransactionFilter {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub category_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub year: Option<i32>,
    pub month: Option<u32>,
}
