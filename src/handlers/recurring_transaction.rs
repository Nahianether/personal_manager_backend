use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use chrono::Utc;
use sqlx::Row;

use crate::models::{RecurringTransaction, CreateRecurringTransactionRequest, UpdateRecurringTransactionRequest};
use crate::services::DbPool;
use crate::middleware::auth::AuthUser;

pub async fn create_recurring_transaction(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Json(request): Json<CreateRecurringTransactionRequest>,
) -> Result<Json<Value>, StatusCode> {
    log::info!("POST /recurring_transactions - Creating recurring transaction for user {}", auth_user.user_id);

    let rt = RecurringTransaction::new(request, auth_user.user_id.clone());
    let start_date_str = rt.start_date.format("%Y-%m-%d %H:%M:%S").to_string();
    let end_date_str = rt.end_date.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string());
    let next_due_date_str = rt.next_due_date.format("%Y-%m-%d %H:%M:%S").to_string();
    let created_at_str = rt.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
    let updated_at_str = rt.updated_at.format("%Y-%m-%d %H:%M:%S").to_string();

    let result = sqlx::query(
        "INSERT INTO recurring_transactions (id, user_id, account_id, transaction_type, amount, currency, category, description, frequency, start_date, end_date, next_due_date, is_active, savings_goal_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&rt.id)
    .bind(&rt.user_id)
    .bind(&rt.account_id)
    .bind(&rt.transaction_type)
    .bind(rt.amount)
    .bind(&rt.currency)
    .bind(&rt.category)
    .bind(&rt.description)
    .bind(&rt.frequency)
    .bind(&start_date_str)
    .bind(&end_date_str)
    .bind(&next_due_date_str)
    .bind(rt.is_active)
    .bind(&rt.savings_goal_id)
    .bind(&created_at_str)
    .bind(&updated_at_str)
    .execute(&pool)
    .await;

    match result {
        Ok(_) => {
            log::info!("Recurring transaction created successfully: {}", rt.id);
            Ok(Json(json!({
                "success": true,
                "data": rt
            })))
        }
        Err(e) => {
            log::error!("Failed to create recurring transaction: {}", e);
            let error_msg = e.to_string();
            if error_msg.contains("UNIQUE constraint failed: recurring_transactions.id") {
                log::warn!("Recurring transaction with ID {} already exists", rt.id);
                Err(StatusCode::CONFLICT)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

pub async fn get_recurring_transactions(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<Value>, StatusCode> {
    log::info!("GET /recurring_transactions - Fetching recurring transactions for user {}", auth_user.user_id);

    let result = sqlx::query(
        "SELECT id, user_id, account_id, transaction_type, amount, currency, category, description, frequency, start_date, end_date, next_due_date, is_active, savings_goal_id, created_at, updated_at FROM recurring_transactions WHERE user_id = ? ORDER BY created_at DESC"
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
                    "accountId": row.get::<String, _>("account_id"),
                    "transactionType": row.get::<String, _>("transaction_type"),
                    "amount": row.get::<f64, _>("amount"),
                    "currency": row.get::<String, _>("currency"),
                    "category": row.get::<Option<String>, _>("category"),
                    "description": row.get::<Option<String>, _>("description"),
                    "frequency": row.get::<String, _>("frequency"),
                    "startDate": row.get::<String, _>("start_date"),
                    "endDate": row.get::<Option<String>, _>("end_date"),
                    "nextDueDate": row.get::<String, _>("next_due_date"),
                    "isActive": row.get::<bool, _>("is_active"),
                    "savingsGoalId": row.get::<Option<String>, _>("savings_goal_id"),
                    "createdAt": row.get::<String, _>("created_at"),
                    "updatedAt": row.get::<String, _>("updated_at")
                })
            }).collect();

            log::info!("Found {} recurring transactions", transactions.len());
            Ok(Json(json!({
                "success": true,
                "data": transactions
            })))
        }
        Err(e) => {
            log::error!("Failed to get recurring transactions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_recurring_transaction(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<Value>, StatusCode> {
    log::info!("GET /recurring_transactions/{} - Fetching recurring transaction by ID", id);

    let result = sqlx::query(
        "SELECT id, user_id, account_id, transaction_type, amount, currency, category, description, frequency, start_date, end_date, next_due_date, is_active, savings_goal_id, created_at, updated_at FROM recurring_transactions WHERE id = ? AND user_id = ?"
    )
    .bind(&id)
    .bind(&auth_user.user_id)
    .fetch_optional(&pool)
    .await;

    match result {
        Ok(Some(row)) => {
            let transaction = json!({
                "id": row.get::<String, _>("id"),
                "userId": row.get::<String, _>("user_id"),
                "accountId": row.get::<String, _>("account_id"),
                "transactionType": row.get::<String, _>("transaction_type"),
                "amount": row.get::<f64, _>("amount"),
                "currency": row.get::<String, _>("currency"),
                "category": row.get::<Option<String>, _>("category"),
                "description": row.get::<Option<String>, _>("description"),
                "frequency": row.get::<String, _>("frequency"),
                "startDate": row.get::<String, _>("start_date"),
                "endDate": row.get::<Option<String>, _>("end_date"),
                "nextDueDate": row.get::<String, _>("next_due_date"),
                "isActive": row.get::<bool, _>("is_active"),
                "savingsGoalId": row.get::<Option<String>, _>("savings_goal_id"),
                "createdAt": row.get::<String, _>("created_at"),
                "updatedAt": row.get::<String, _>("updated_at")
            });

            Ok(Json(json!({
                "success": true,
                "data": transaction
            })))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            log::error!("Failed to get recurring transaction: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_recurring_transaction(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Json(request): Json<UpdateRecurringTransactionRequest>,
) -> Result<Json<Value>, StatusCode> {
    log::info!("PUT /recurring_transactions/{} - Updating recurring transaction", id);

    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let start_date_str = request.start_date.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string());
    let end_date_str = request.end_date.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string());
    let next_due_date_str = request.next_due_date.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string());

    let result = sqlx::query(
        "UPDATE recurring_transactions SET account_id = COALESCE(?, account_id), transaction_type = COALESCE(?, transaction_type), amount = COALESCE(?, amount), currency = COALESCE(?, currency), category = COALESCE(?, category), description = COALESCE(?, description), frequency = COALESCE(?, frequency), start_date = COALESCE(?, start_date), end_date = COALESCE(?, end_date), next_due_date = COALESCE(?, next_due_date), is_active = COALESCE(?, is_active), savings_goal_id = COALESCE(?, savings_goal_id), updated_at = ? WHERE id = ? AND user_id = ?"
    )
    .bind(request.account_id)
    .bind(request.transaction_type)
    .bind(request.amount)
    .bind(request.currency)
    .bind(request.category)
    .bind(request.description)
    .bind(request.frequency)
    .bind(start_date_str)
    .bind(end_date_str)
    .bind(next_due_date_str)
    .bind(request.is_active)
    .bind(request.savings_goal_id)
    .bind(&now)
    .bind(&id)
    .bind(&auth_user.user_id)
    .execute(&pool)
    .await;

    match result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                Err(StatusCode::NOT_FOUND)
            } else {
                log::info!("Recurring transaction updated successfully: {}", id);
                Ok(Json(json!({
                    "success": true,
                    "message": "Recurring transaction updated successfully"
                })))
            }
        }
        Err(e) => {
            log::error!("Failed to update recurring transaction: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_recurring_transaction(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<Value>, StatusCode> {
    log::info!("DELETE /recurring_transactions/{} - Deleting recurring transaction", id);

    let result = sqlx::query("DELETE FROM recurring_transactions WHERE id = ? AND user_id = ?")
        .bind(&id)
        .bind(&auth_user.user_id)
        .execute(&pool)
        .await;

    match result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                Err(StatusCode::NOT_FOUND)
            } else {
                log::info!("Recurring transaction deleted successfully: {}", id);
                Ok(Json(json!({
                    "success": true,
                    "message": "Recurring transaction deleted successfully"
                })))
            }
        }
        Err(e) => {
            log::error!("Failed to delete recurring transaction: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
