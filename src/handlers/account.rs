use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use chrono::Utc;
use sqlx::Row;

use crate::models::{Account, CreateAccountRequest, UpdateAccountRequest};
use crate::services::DbPool;

pub async fn create_account(
    State(pool): State<DbPool>,
    Json(request): Json<CreateAccountRequest>,
) -> Result<Json<Value>, StatusCode> {
    log::info!("📥 POST /accounts - Creating new account: {}", request.name);
    log::debug!("Request payload: {:?}", request);
    
    let account = Account::new(request);
    let account_type_str = format!("{:?}", account.account_type).to_lowercase();
    let created_at_str = account.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
    let updated_at_str = account.updated_at.format("%Y-%m-%d %H:%M:%S").to_string();
    
    log::debug!("Generated account ID: {}", account.id);
    
    // Use INSERT OR REPLACE to handle duplicate IDs
    let result = sqlx::query(
        "INSERT OR REPLACE INTO accounts (id, name, account_type, balance, currency, credit_limit, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&account.id)
    .bind(&account.name)
    .bind(&account_type_str)
    .bind(account.balance)
    .bind(&account.currency)
    .bind(account.credit_limit)
    .bind(&created_at_str)
    .bind(&updated_at_str)
    .execute(&pool)
    .await;

    match result {
        Ok(_) => {
            log::info!("✅ Account created/updated successfully - ID: {}, Name: {}", account.id, account.name);
            Ok(Json(json!({
                "success": true,
                "data": account
            })))
        },
        Err(e) => {
            log::error!("❌ Failed to create account '{}': {}", account.name, e);
            log::error!("Database error details: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_accounts(
    State(pool): State<DbPool>,
) -> Result<Json<Value>, StatusCode> {
    log::info!("📥 GET /accounts - Fetching all accounts");
    
    let result = sqlx::query(
        "SELECT id, name, account_type, balance, currency, credit_limit, created_at, updated_at FROM accounts ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await;

    match result {
        Ok(rows) => {
            let accounts: Vec<_> = rows.into_iter().map(|row| {
                json!({
                    "id": row.get::<String, _>("id"),
                    "name": row.get::<String, _>("name"),
                    "account_type": row.get::<String, _>("account_type"),
                    "balance": row.get::<f64, _>("balance"),
                    "currency": row.get::<String, _>("currency"),
                    "credit_limit": row.get::<Option<f64>, _>("credit_limit"),
                    "created_at": row.get::<String, _>("created_at"),
                    "updated_at": row.get::<String, _>("updated_at")
                })
            }).collect();
            
            log::info!("✅ Found {} accounts", accounts.len());
            Ok(Json(json!({
                "success": true,
                "data": accounts
            })))
        }
        Err(e) => {
            log::error!("❌ Failed to get accounts: {}", e);
            log::error!("Database error details: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_account(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
) -> Result<Json<Value>, StatusCode> {
    log::info!("📥 GET /accounts/{} - Fetching account by ID", id);
    
    let result = sqlx::query(
        "SELECT id, name, account_type, balance, currency, credit_limit, created_at, updated_at FROM accounts WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await;

    match result {
        Ok(Some(row)) => {
            let account_name = row.get::<String, _>("name");
            let account = json!({
                "id": row.get::<String, _>("id"),
                "name": account_name,
                "account_type": row.get::<String, _>("account_type"),
                "balance": row.get::<f64, _>("balance"),
                "currency": row.get::<String, _>("currency"),
                "credit_limit": row.get::<Option<f64>, _>("credit_limit"),
                "created_at": row.get::<String, _>("created_at"),
                "updated_at": row.get::<String, _>("updated_at")
            });
            
            log::info!("✅ Found account: {}", account_name);
            Ok(Json(json!({
                "success": true,
                "data": account
            })))
        }
        Ok(None) => {
            log::warn!("⚠️  Account not found with ID: {}", id);
            Err(StatusCode::NOT_FOUND)
        },
        Err(e) => {
            log::error!("❌ Failed to get account {}: {}", id, e);
            log::error!("Database error details: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_account(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
    Json(request): Json<UpdateAccountRequest>,
) -> Result<Json<Value>, StatusCode> {
    log::info!("📥 PUT /accounts/{} - Updating account", id);
    log::debug!("Update request: {:?}", request);
    
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let account_type_str = request.account_type.map(|t| format!("{:?}", t).to_lowercase());
    
    let result = sqlx::query(
        "UPDATE accounts SET name = COALESCE(?, name), account_type = COALESCE(?, account_type), balance = COALESCE(?, balance), currency = COALESCE(?, currency), credit_limit = COALESCE(?, credit_limit), updated_at = ? WHERE id = ?"
    )
    .bind(request.name.as_ref())
    .bind(account_type_str)
    .bind(request.balance)
    .bind(request.currency.as_ref())
    .bind(request.credit_limit)
    .bind(&now)
    .bind(&id)
    .execute(&pool)
    .await;

    match result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                log::warn!("⚠️  Account not found for update: {}", id);
                Err(StatusCode::NOT_FOUND)
            } else {
                log::info!("✅ Account updated successfully: {}", id);
                Ok(Json(json!({
                    "success": true,
                    "message": "Account updated successfully"
                })))
            }
        }
        Err(e) => {
            log::error!("❌ Failed to update account {}: {}", id, e);
            log::error!("Database error details: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_account(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
) -> Result<Json<Value>, StatusCode> {
    log::info!("📥 DELETE /accounts/{} - Deleting account", id);
    
    let result = sqlx::query("DELETE FROM accounts WHERE id = ?")
        .bind(&id)
        .execute(&pool)
        .await;

    match result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                log::warn!("⚠️  Account not found for deletion: {}", id);
                Err(StatusCode::NOT_FOUND)
            } else {
                log::info!("✅ Account deleted successfully: {}", id);
                Ok(Json(json!({
                    "success": true,
                    "message": "Account deleted successfully"
                })))
            }
        }
        Err(e) => {
            log::error!("❌ Failed to delete account {}: {}", id, e);
            log::error!("Database error details: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}