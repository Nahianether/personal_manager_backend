use sqlx::{sqlite::SqlitePool, Pool, Sqlite};
use anyhow::Result;

pub type DbPool = Pool<Sqlite>;

pub async fn init_db(database_url: &str) -> Result<DbPool> {
    // Create database connection pool
    let pool = SqlitePool::connect(database_url).await?;
    
    // Enable foreign key constraints for SQLite
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await?;
    
    log::info!("✅ Database connected successfully");
    Ok(pool)
}

pub async fn create_tables(pool: &DbPool) -> Result<()> {
    // Create accounts table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS accounts (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            name TEXT NOT NULL,
            account_type TEXT NOT NULL,
            balance REAL NOT NULL,
            currency TEXT NOT NULL DEFAULT 'BDT',
            credit_limit REAL,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create categories table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS categories (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category_type TEXT NOT NULL,
            icon TEXT NOT NULL,
            color TEXT NOT NULL,
            is_default BOOLEAN NOT NULL DEFAULT FALSE,
            created_at DATETIME NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create transactions table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS transactions (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            account_id TEXT NOT NULL,
            transaction_type TEXT NOT NULL,
            amount REAL NOT NULL,
            currency TEXT NOT NULL DEFAULT 'BDT',
            category TEXT,
            description TEXT,
            date DATETIME NOT NULL,
            created_at DATETIME NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create liabilities table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS liabilities (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            person_name TEXT NOT NULL,
            amount REAL NOT NULL,
            currency TEXT NOT NULL DEFAULT 'BDT',
            due_date DATETIME NOT NULL,
            is_paid BOOLEAN NOT NULL DEFAULT FALSE,
            description TEXT,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create loans table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS loans (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            person_name TEXT NOT NULL,
            amount REAL NOT NULL,
            currency TEXT NOT NULL DEFAULT 'BDT',
            loan_date DATETIME NOT NULL,
            return_date DATETIME,
            is_returned BOOLEAN NOT NULL DEFAULT FALSE,
            description TEXT,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create users table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    log::info!("✅ All database tables created successfully");
    Ok(())
}