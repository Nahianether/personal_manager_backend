use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use sqlx::Row;

use crate::models::{Transaction, CreateTransactionRequest, UpdateTransactionRequest};
use crate::services::DbPool;

pub async fn create_transaction(
    State(pool): State<DbPool>,
    Json(request): Json<CreateTransactionRequest>,
) -> Result<Json<Value>, StatusCode> {
    let transaction = Transaction::new(request);
    let transaction_type_str = format!("{:?}", transaction.transaction_type).to_lowercase();
    let date_str = transaction.date.format("%Y-%m-%d %H:%M:%S").to_string();
    let created_at_str = transaction.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
    
    let result = sqlx::query(
        "INSERT INTO transactions (id, account_id, transaction_type, amount, currency, category, description, date, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&transaction.id)
    .bind(&transaction.account_id)
    .bind(&transaction_type_str)
    .bind(transaction.amount)
    .bind(&transaction.currency)
    .bind(&transaction.category)
    .bind(&transaction.description)
    .bind(&date_str)
    .bind(&created_at_str)
    .execute(&pool)
    .await;

    match result {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "data": transaction
        }))),
        Err(e) => {
            log::error!("Failed to create transaction: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_transactions(
    State(pool): State<DbPool>,
) -> Result<Json<Value>, StatusCode> {
    let result = sqlx::query(
        "SELECT id, account_id, transaction_type, amount, currency, category, description, date, created_at FROM transactions ORDER BY date DESC"
    )
    .fetch_all(&pool)
    .await;

    match result {
        Ok(rows) => {
            let transactions: Vec<_> = rows.into_iter().map(|row| {
                json!({
                    "id": row.get::<String, _>("id"),
                    "account_id": row.get::<String, _>("account_id"),
                    "transaction_type": row.get::<String, _>("transaction_type"),
                    "amount": row.get::<f64, _>("amount"),
                    "currency": row.get::<String, _>("currency"),
                    "category": row.get::<String, _>("category"),
                    "description": row.get::<Option<String>, _>("description"),
                    "date": row.get::<String, _>("date"),
                    "created_at": row.get::<String, _>("created_at")
                })
            }).collect();
            
            Ok(Json(json!({
                "success": true,
                "data": transactions
            })))
        }
        Err(e) => {
            log::error!("Failed to get transactions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_transaction(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
) -> Result<Json<Value>, StatusCode> {
    let result = sqlx::query(
        "SELECT id, account_id, transaction_type, amount, currency, category, description, date, created_at FROM transactions WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await;

    match result {
        Ok(Some(row)) => {
            let transaction = json!({
                "id": row.get::<String, _>("id"),
                "account_id": row.get::<String, _>("account_id"),
                "transaction_type": row.get::<String, _>("transaction_type"),
                "amount": row.get::<f64, _>("amount"),
                "currency": row.get::<String, _>("currency"),
                "category": row.get::<String, _>("category"),
                "description": row.get::<Option<String>, _>("description"),
                "date": row.get::<String, _>("date"),
                "created_at": row.get::<String, _>("created_at")
            });
            
            Ok(Json(json!({
                "success": true,
                "data": transaction
            })))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            log::error!("Failed to get transaction: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_transaction(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
    Json(request): Json<UpdateTransactionRequest>,
) -> Result<Json<Value>, StatusCode> {
    let transaction_type_str = request.transaction_type.map(|t| format!("{:?}", t).to_lowercase());
    let date_str = request.date.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string());
    
    let result = sqlx::query(
        "UPDATE transactions SET account_id = COALESCE(?, account_id), transaction_type = COALESCE(?, transaction_type), amount = COALESCE(?, amount), currency = COALESCE(?, currency), category = COALESCE(?, category), description = COALESCE(?, description), date = COALESCE(?, date) WHERE id = ?"
    )
    .bind(request.account_id)
    .bind(transaction_type_str)
    .bind(request.amount)
    .bind(request.currency)
    .bind(request.category)
    .bind(request.description)
    .bind(date_str)
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
                    "message": "Transaction updated successfully"
                })))
            }
        }
        Err(e) => {
            log::error!("Failed to update transaction: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_transaction(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
) -> Result<Json<Value>, StatusCode> {
    let result = sqlx::query("DELETE FROM transactions WHERE id = ?")
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
                    "message": "Transaction deleted successfully"
                })))
            }
        }
        Err(e) => {
            log::error!("Failed to delete transaction: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}