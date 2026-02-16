use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use sqlx::Row;

use crate::models::{Transaction, CreateTransactionRequest, UpdateTransactionRequest};
use crate::services::DbPool;
use crate::middleware::auth::AuthUser;

pub async fn create_transaction(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Json(request): Json<CreateTransactionRequest>,
) -> Result<Json<Value>, StatusCode> {
    log::info!("📥 POST /transactions - Creating transaction for user {}", auth_user.user_id);
    log::info!("✅ Successfully parsed request: {:?}", request);

    let transaction = Transaction::new(request.clone(), auth_user.user_id.clone());
    let transaction_type_str = format!("{:?}", transaction.transaction_type).to_lowercase();
    let date_str = transaction.date.format("%Y-%m-%d %H:%M:%S").to_string();
    let created_at_str = transaction.created_at.format("%Y-%m-%d %H:%M:%S").to_string();

    let result = sqlx::query(
        "INSERT INTO transactions (id, user_id, account_id, transaction_type, amount, currency, category, description, date, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&transaction.id)
    .bind(&transaction.user_id)
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
        Ok(_) => {
            log::info!("✅ Transaction created successfully: {} {} ({})", transaction.amount, transaction.currency, transaction.id);
            Ok(Json(json!({
                "success": true,
                "data": transaction
            })))
        }
        Err(e) => {
            log::error!("❌ Failed to create transaction: {}", e);
            log::error!("Database error details: {:?}", e);
            log::error!("Raw request data: {:?}", request);

            let error_msg = e.to_string();
            if error_msg.contains("UNIQUE constraint failed: transactions.id") {
                log::warn!("⚠️  Transaction with ID {} already exists", transaction.id);
                Err(StatusCode::CONFLICT)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

pub async fn get_transactions(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<Value>, StatusCode> {
    log::info!("📥 GET /transactions - Fetching transactions for user {}", auth_user.user_id);

    let result = sqlx::query(
        "SELECT id, user_id, account_id, transaction_type, amount, currency, category, description, date, created_at FROM transactions WHERE user_id = ? ORDER BY date DESC"
    )
    .bind(&auth_user.user_id)
    .fetch_all(&pool)
    .await;

    match result {
        Ok(rows) => {
            let transactions: Vec<_> = rows.into_iter().map(|row| {
                json!({
                    "id": row.get::<String, _>("id"),
                    "userId": row.get::<String, _>("user_id"),
                    "account_id": row.get::<String, _>("account_id"),
                    "transaction_type": row.get::<String, _>("transaction_type"),
                    "amount": row.get::<f64, _>("amount"),
                    "currency": row.get::<String, _>("currency"),
                    "category": row.get::<Option<String>, _>("category"),
                    "description": row.get::<Option<String>, _>("description"),
                    "date": row.get::<String, _>("date"),
                    "created_at": row.get::<String, _>("created_at")
                })
            }).collect();

            log::info!("✅ Found {} transactions", transactions.len());
            Ok(Json(json!({
                "success": true,
                "data": transactions
            })))
        }
        Err(e) => {
            log::error!("❌ Failed to get transactions: {}", e);
            log::error!("Database error details: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_transaction(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<Value>, StatusCode> {
    log::info!("📥 GET /transactions/{} - Fetching transaction by ID", id);

    let result = sqlx::query(
        "SELECT id, user_id, account_id, transaction_type, amount, currency, category, description, date, created_at FROM transactions WHERE id = ? AND user_id = ?"
    )
    .bind(&id)
    .bind(&auth_user.user_id)
    .fetch_optional(&pool)
    .await;

    match result {
        Ok(Some(row)) => {
            let amount = row.get::<f64, _>("amount");
            let currency = row.get::<String, _>("currency");
            let transaction = json!({
                "id": row.get::<String, _>("id"),
                "userId": row.get::<String, _>("user_id"),
                "account_id": row.get::<String, _>("account_id"),
                "transaction_type": row.get::<String, _>("transaction_type"),
                "amount": amount,
                "currency": currency,
                "category": row.get::<Option<String>, _>("category"),
                "description": row.get::<Option<String>, _>("description"),
                "date": row.get::<String, _>("date"),
                "created_at": row.get::<String, _>("created_at")
            });

            log::info!("✅ Found transaction: {} {}", amount, currency);
            Ok(Json(json!({
                "success": true,
                "data": transaction
            })))
        }
        Ok(None) => {
            log::warn!("⚠️  Transaction not found with ID: {}", id);
            Err(StatusCode::NOT_FOUND)
        },
        Err(e) => {
            log::error!("❌ Failed to get transaction {}: {}", id, e);
            log::error!("Database error details: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_transaction(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Json(request): Json<UpdateTransactionRequest>,
) -> Result<Json<Value>, StatusCode> {
    log::info!("📥 PUT /transactions/{} - Updating transaction", id);
    log::debug!("Update request: {:?}", request);

    let transaction_type_str = request.transaction_type.map(|t| format!("{:?}", t).to_lowercase());
    let date_str = request.date.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string());

    let result = sqlx::query(
        "UPDATE transactions SET account_id = COALESCE(?, account_id), transaction_type = COALESCE(?, transaction_type), amount = COALESCE(?, amount), currency = COALESCE(?, currency), category = COALESCE(?, category), description = COALESCE(?, description), date = COALESCE(?, date) WHERE id = ? AND user_id = ?"
    )
    .bind(request.account_id)
    .bind(transaction_type_str)
    .bind(request.amount)
    .bind(request.currency)
    .bind(request.category)
    .bind(request.description)
    .bind(date_str)
    .bind(&id)
    .bind(&auth_user.user_id)
    .execute(&pool)
    .await;

    match result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                log::warn!("⚠️  Transaction not found for update: {}", id);
                Err(StatusCode::NOT_FOUND)
            } else {
                log::info!("✅ Transaction updated successfully: {}", id);
                Ok(Json(json!({
                    "success": true,
                    "message": "Transaction updated successfully"
                })))
            }
        }
        Err(e) => {
            log::error!("❌ Failed to update transaction {}: {}", id, e);
            log::error!("Database error details: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_transaction(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<Value>, StatusCode> {
    log::info!("📥 DELETE /transactions/{} - Deleting transaction", id);

    let result = sqlx::query("DELETE FROM transactions WHERE id = ? AND user_id = ?")
        .bind(&id)
        .bind(&auth_user.user_id)
        .execute(&pool)
        .await;

    match result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                log::warn!("⚠️  Transaction not found for deletion: {}", id);
                Err(StatusCode::NOT_FOUND)
            } else {
                log::info!("✅ Transaction deleted successfully: {}", id);
                Ok(Json(json!({
                    "success": true,
                    "message": "Transaction deleted successfully"
                })))
            }
        }
        Err(e) => {
            log::error!("❌ Failed to delete transaction {}: {}", id, e);
            log::error!("Database error details: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
