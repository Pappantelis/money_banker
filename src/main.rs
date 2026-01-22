use sqlx::postgres::PgPoolOptions;
use anyhow::Result;
use dotenvy::dotenv;
use std::env;
use rust_decimal::Decimal;
use chrono::NaiveDateTime;

// Struct που αντιστοιχεί στον πίνακα
#[derive(Debug, sqlx::FromRow)]
struct TestExpense {
    id: i32,
    description: String,
    amount: Decimal,
    created_at: Option<NaiveDateTime>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file
    dotenv().ok();
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");

    println!("🔌 Connecting to database...");
    
    // Create connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    
    println!("✅ Connected successfully!\n");

    // Query all test expenses
    println!("📊 Fetching test expenses...\n");
    
    let expenses = sqlx::query_as::<_, TestExpense>(
        "SELECT * FROM test_expenses ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await?;

    // Print them
    for expense in expenses {
        println!("💰 {} - €{} (ID: {})", 
            expense.description, 
            expense.amount,
            expense.id
        );
    }

    println!("\n✨ Success!");
    
    Ok(())
}