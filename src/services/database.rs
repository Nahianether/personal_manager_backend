use sqlx::{sqlite::{SqlitePool, SqliteConnectOptions, SqliteJournalMode}, Pool, Sqlite};
use anyhow::Result;
use std::str::FromStr;

pub type DbPool = Pool<Sqlite>;

pub async fn init_db(database_url: &str) -> Result<DbPool> {
    // Create database connection pool with create_if_missing
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal);
    let pool = SqlitePool::connect_with(options).await?;

    // Enable foreign key constraints for SQLite
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await?;

    log::info!("✅ Database connected successfully");
    Ok(pool)
}

pub async fn create_tables(pool: &DbPool) -> Result<()> {
    // Create users table first (referenced by other tables)
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
            is_historical_entry BOOLEAN NOT NULL DEFAULT FALSE,
            account_id TEXT,
            transaction_id TEXT,
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
            is_historical_entry BOOLEAN NOT NULL DEFAULT FALSE,
            account_id TEXT,
            transaction_id TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Migrations for existing databases: add new columns if they don't exist
    // .ok() ignores "duplicate column" errors for databases that already have these columns
    sqlx::query("ALTER TABLE loans ADD COLUMN is_historical_entry BOOLEAN NOT NULL DEFAULT FALSE").execute(pool).await.ok();
    sqlx::query("ALTER TABLE loans ADD COLUMN account_id TEXT").execute(pool).await.ok();
    sqlx::query("ALTER TABLE loans ADD COLUMN transaction_id TEXT").execute(pool).await.ok();

    sqlx::query("ALTER TABLE liabilities ADD COLUMN is_historical_entry BOOLEAN NOT NULL DEFAULT FALSE").execute(pool).await.ok();
    sqlx::query("ALTER TABLE liabilities ADD COLUMN account_id TEXT").execute(pool).await.ok();
    sqlx::query("ALTER TABLE liabilities ADD COLUMN transaction_id TEXT").execute(pool).await.ok();

    log::info!("✅ All database tables created successfully");
    Ok(())
}
