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
    let account = Account::new(request);
    let account_type_str = format!("{:?}", account.account_type).to_lowercase();
    let created_at_str = account.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
    let updated_at_str = account.updated_at.format("%Y-%m-%d %H:%M:%S").to_string();
    
    let result = sqlx::query(
        "INSERT INTO accounts (id, name, account_type, balance, currency, credit_limit, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
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
        Ok(_) => Ok(Json(json!({
            "success": true,
            "data": account
        }))),
        Err(e) => {
            log::error!("Failed to create account: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_accounts(
    State(pool): State<DbPool>,
) -> Result<Json<Value>, StatusCode> {
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
            
            Ok(Json(json!({
                "success": true,
                "data": accounts
            })))
        }
        Err(e) => {
            log::error!("Failed to get accounts: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_account(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
) -> Result<Json<Value>, StatusCode> {
    let result = sqlx::query(
        "SELECT id, name, account_type, balance, currency, credit_limit, created_at, updated_at FROM accounts WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await;

    match result {
        Ok(Some(row)) => {
            let account = json!({
                "id": row.get::<String, _>("id"),
                "name": row.get::<String, _>("name"),
                "account_type": row.get::<String, _>("account_type"),
                "balance": row.get::<f64, _>("balance"),
                "currency": row.get::<String, _>("currency"),
                "credit_limit": row.get::<Option<f64>, _>("credit_limit"),
                "created_at": row.get::<String, _>("created_at"),
                "updated_at": row.get::<String, _>("updated_at")
            });
            
            Ok(Json(json!({
                "success": true,
                "data": account
            })))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            log::error!("Failed to get account: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_account(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
    Json(request): Json<UpdateAccountRequest>,
) -> Result<Json<Value>, StatusCode> {
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let account_type_str = request.account_type.map(|t| format!("{:?}", t).to_lowercase());
    
    let result = sqlx::query(
        "UPDATE accounts SET name = COALESCE(?, name), account_type = COALESCE(?, account_type), balance = COALESCE(?, balance), currency = COALESCE(?, currency), credit_limit = COALESCE(?, credit_limit), updated_at = ? WHERE id = ?"
    )
    .bind(request.name)
    .bind(account_type_str)
    .bind(request.balance)
    .bind(request.currency)
    .bind(request.credit_limit)
    .bind(&now)
    .bind(&id)
    .execute(&pool)
    .await;

    match result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                Err(StatusCode::NOT_FOUND)
            } else {
                Ok(Json(json!({
                    "success": true,
                    "message": "Account updated successfully"
                })))
            }
        }
        Err(e) => {
            log::error!("Failed to update account: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_account(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
) -> Result<Json<Value>, StatusCode> {
    let result = sqlx::query("DELETE FROM accounts WHERE id = ?")
        .bind(&id)
        .execute(&pool)
        .await;

    match result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                Err(StatusCode::NOT_FOUND)
            } else {
                Ok(Json(json!({
                    "success": true,
                    "message": "Account deleted successfully"
                })))
            }
        }
        Err(e) => {
            log::error!("Failed to delete account: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}