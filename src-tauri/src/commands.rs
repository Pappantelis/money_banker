use crate::{
    auth,
    config::Settings,
    models::{Category, Transaction, User},
    services::{CategoryService, TransactionService},
    state::AppState,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tauri::State;

/// Filter for querying transactions
#[derive(Debug, Deserialize)]
pub struct TransactionFilter {
    pub year: Option<i32>,
    pub month: Option<u32>,
    pub category_id: Option<String>,
}

/// Data for creating a new transaction
#[derive(Debug, Deserialize)]
pub struct CreateTransactionInput {
    pub amount: f64,
    pub store: Option<String>,
    pub description: Option<String>,
    pub category_id: Option<String>,
    pub transaction_date: String,
    pub is_income: bool,
}

/// Monthly summary response
#[derive(Debug, Serialize)]
pub struct MonthlySummary {
    pub income: f64,
    pub expenses: f64,
    pub balance: f64,
    pub transaction_count: i64,
}

/// Get the current logged-in user
#[tauri::command]
pub async fn get_current_user(state: State<'_, AppState>) -> Result<User, String> {
    state
        .get_user()
        .await
        .ok_or_else(|| "No user logged in".to_string())
}

/// Perform Google OAuth login
#[tauri::command]
pub async fn login(state: State<'_, AppState>) -> Result<User, String> {
    let settings = Settings::load().map_err(|e| e.to_string())?;

    let user = auth::perform_google_login(&settings, &state.pool)
        .await
        .map_err(|e| e.to_string())?;

    state.set_user(Some(user.clone())).await;

    // Create default categories for new user if needed
    let category_service = CategoryService::new(state.pool.clone());
    let categories = category_service
        .get_user_categories(user.id)
        .await
        .map_err(|e| e.to_string())?;

    if categories.is_empty() {
        category_service
            .create_default_categories(user.id)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(user)
}

/// Logout the current user
#[tauri::command]
pub async fn logout(state: State<'_, AppState>) -> Result<(), String> {
    auth::logout().await.map_err(|e| e.to_string())?;
    state.set_user(None).await;
    Ok(())
}

/// Get all categories for the current user
#[tauri::command]
pub async fn get_categories(state: State<'_, AppState>) -> Result<Vec<Category>, String> {
    let user = state
        .get_user()
        .await
        .ok_or_else(|| "No user logged in".to_string())?;

    let service = CategoryService::new(state.pool.clone());
    service
        .get_user_categories(user.id)
        .await
        .map_err(|e| e.to_string())
}

/// Get transactions with optional filtering
#[tauri::command]
pub async fn get_transactions(
    state: State<'_, AppState>,
    filter: TransactionFilter,
) -> Result<Vec<Transaction>, String> {
    let user = state
        .get_user()
        .await
        .ok_or_else(|| "No user logged in".to_string())?;

    let service = TransactionService::new(state.pool.clone());

    // If year and month are specified, filter by month
    if let (Some(year), Some(month)) = (filter.year, filter.month) {
        service
            .get_user_transactions_by_month(user.id, year, month)
            .await
            .map_err(|e| e.to_string())
    } else {
        // Get all transactions (limited)
        service
            .get_user_transactions(user.id, 100)
            .await
            .map_err(|e| e.to_string())
    }
}

/// Add a new transaction
#[tauri::command]
pub async fn add_transaction(
    state: State<'_, AppState>,
    transaction: CreateTransactionInput,
) -> Result<Transaction, String> {
    let user = state
        .get_user()
        .await
        .ok_or_else(|| "No user logged in".to_string())?;

    let service = TransactionService::new(state.pool.clone());

    // Parse the date
    let date = NaiveDate::parse_from_str(&transaction.transaction_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format: {}", e))?;

    // Parse category_id as UUID if present
    let category_id = transaction
        .category_id
        .map(|id| {
            uuid::Uuid::parse_str(&id).map_err(|e| format!("Invalid category ID: {}", e))
        })
        .transpose()?;

    // Convert amount to Decimal
    let amount = Decimal::try_from(transaction.amount)
        .map_err(|e| format!("Invalid amount: {}", e))?;

    let tx = service
        .create_transaction(
            user.id,
            amount,
            transaction.store,
            transaction.description,
            category_id,
            date,
            "manual".to_string(),
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(tx)
}

/// Get monthly summary (income, expenses, balance)
#[tauri::command]
pub async fn get_monthly_summary(
    state: State<'_, AppState>,
    year: i32,
    month: u32,
) -> Result<MonthlySummary, String> {
    let user = state
        .get_user()
        .await
        .ok_or_else(|| "No user logged in".to_string())?;

    let service = TransactionService::new(state.pool.clone());

    let transactions = service
        .get_user_transactions_by_month(user.id, year, month)
        .await
        .map_err(|e| e.to_string())?;

    let mut income = Decimal::ZERO;
    let mut expenses = Decimal::ZERO;
    let transaction_count = transactions.len() as i64;

    for tx in &transactions {
        if tx.amount >= Decimal::ZERO {
            income += tx.amount;
        } else {
            expenses += tx.amount.abs();
        }
    }

    Ok(MonthlySummary {
        income: income.to_string().parse().unwrap_or(0.0),
        expenses: expenses.to_string().parse().unwrap_or(0.0),
        balance: (income - expenses).to_string().parse().unwrap_or(0.0),
        transaction_count,
    })
}
