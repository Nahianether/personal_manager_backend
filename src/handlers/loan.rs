use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use chrono::Utc;
use sqlx::Row;

use crate::models::{Loan, CreateLoanRequest, UpdateLoanRequest};
use crate::services::DbPool;
use crate::middleware::auth::AuthUser;

pub async fn create_loan(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Json(request): Json<CreateLoanRequest>,
) -> Result<Json<Value>, StatusCode> {
    log::info!("📥 POST /loans - Creating loan for user {}", auth_user.user_id);

    let loan = Loan::new(request, auth_user.user_id.clone());
    let loan_date_str = loan.loan_date.format("%Y-%m-%d %H:%M:%S").to_string();
    let return_date_str = loan.return_date.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string());
    let created_at_str = loan.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
    let updated_at_str = loan.updated_at.format("%Y-%m-%d %H:%M:%S").to_string();

    let result = sqlx::query(
        "INSERT INTO loans (id, user_id, person_name, amount, currency, loan_date, return_date, is_returned, description, created_at, updated_at, is_historical_entry, account_id, transaction_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&loan.id)
    .bind(&loan.user_id)
    .bind(&loan.person_name)
    .bind(loan.amount)
    .bind(&loan.currency)
    .bind(&loan_date_str)
    .bind(&return_date_str)
    .bind(loan.is_returned)
    .bind(&loan.description)
    .bind(&created_at_str)
    .bind(&updated_at_str)
    .bind(loan.is_historical_entry)
    .bind(&loan.account_id)
    .bind(&loan.transaction_id)
    .execute(&pool)
    .await;

    match result {
        Ok(_) => {
            log::info!("✅ Loan created successfully: {} ({})", loan.person_name, loan.id);
            Ok(Json(json!({
                "success": true,
                "data": loan
            })))
        }
        Err(e) => {
            log::error!("❌ Failed to create loan: {}", e);
            let error_msg = e.to_string();
            if error_msg.contains("UNIQUE constraint failed: loans.id") {
                log::warn!("⚠️  Loan with ID {} already exists", loan.id);
                Err(StatusCode::CONFLICT)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

pub async fn get_loans(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<Value>, StatusCode> {
    log::info!("📥 GET /loans - Fetching loans for user {}", auth_user.user_id);

    let result = sqlx::query(
        "SELECT id, user_id, person_name, amount, currency, loan_date, return_date, is_returned, description, created_at, updated_at, is_historical_entry, account_id, transaction_id FROM loans WHERE user_id = ? ORDER BY loan_date DESC"
    )
    .bind(&auth_user.user_id)
    .fetch_all(&pool)
    .await;

    match result {
        Ok(rows) => {
            let loans: Vec<_> = rows.into_iter().map(|row| {
                json!({
                    "id": row.get::<String, _>("id"),
                    "user_id": row.get::<String, _>("user_id"),
                    "person_name": row.get::<String, _>("person_name"),
                    "amount": row.get::<f64, _>("amount"),
                    "currency": row.get::<String, _>("currency"),
                    "loan_date": row.get::<String, _>("loan_date"),
                    "return_date": row.get::<Option<String>, _>("return_date"),
                    "is_returned": row.get::<bool, _>("is_returned"),
                    "description": row.get::<Option<String>, _>("description"),
                    "created_at": row.get::<String, _>("created_at"),
                    "updated_at": row.get::<String, _>("updated_at"),
                    "is_historical_entry": row.get::<bool, _>("is_historical_entry"),
                    "account_id": row.get::<Option<String>, _>("account_id"),
                    "transaction_id": row.get::<Option<String>, _>("transaction_id")
                })
            }).collect();

            log::info!("✅ Found {} loans", loans.len());
            Ok(Json(json!({
                "success": true,
                "data": loans
            })))
        }
        Err(e) => {
            log::error!("Failed to get loans: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_loan(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<Value>, StatusCode> {
    log::info!("📥 GET /loans/{} - Fetching loan by ID", id);

    let result = sqlx::query(
        "SELECT id, user_id, person_name, amount, currency, loan_date, return_date, is_returned, description, created_at, updated_at, is_historical_entry, account_id, transaction_id FROM loans WHERE id = ? AND user_id = ?"
    )
    .bind(&id)
    .bind(&auth_user.user_id)
    .fetch_optional(&pool)
    .await;

    match result {
        Ok(Some(row)) => {
            let loan = json!({
                "id": row.get::<String, _>("id"),
                "user_id": row.get::<String, _>("user_id"),
                "person_name": row.get::<String, _>("person_name"),
                "amount": row.get::<f64, _>("amount"),
                "currency": row.get::<String, _>("currency"),
                "loan_date": row.get::<String, _>("loan_date"),
                "return_date": row.get::<Option<String>, _>("return_date"),
                "is_returned": row.get::<bool, _>("is_returned"),
                "description": row.get::<Option<String>, _>("description"),
                "created_at": row.get::<String, _>("created_at"),
                "updated_at": row.get::<String, _>("updated_at"),
                "is_historical_entry": row.get::<bool, _>("is_historical_entry"),
                "account_id": row.get::<Option<String>, _>("account_id"),
                "transaction_id": row.get::<Option<String>, _>("transaction_id")
            });

            Ok(Json(json!({
                "success": true,
                "data": loan
            })))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            log::error!("Failed to get loan: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_loan(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Json(request): Json<UpdateLoanRequest>,
) -> Result<Json<Value>, StatusCode> {
    log::info!("📥 PUT /loans/{} - Updating loan", id);

    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let loan_date_str = request.loan_date.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string());
    let return_date_str = request.return_date.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string());

    let result = sqlx::query(
        "UPDATE loans SET person_name = COALESCE(?, person_name), amount = COALESCE(?, amount), currency = COALESCE(?, currency), loan_date = COALESCE(?, loan_date), return_date = COALESCE(?, return_date), is_returned = COALESCE(?, is_returned), description = COALESCE(?, description), is_historical_entry = COALESCE(?, is_historical_entry), account_id = COALESCE(?, account_id), transaction_id = COALESCE(?, transaction_id), updated_at = ? WHERE id = ? AND user_id = ?"
    )
    .bind(request.person_name)
    .bind(request.amount)
    .bind(request.currency)
    .bind(loan_date_str)
    .bind(return_date_str)
    .bind(request.is_returned)
    .bind(request.description)
    .bind(request.is_historical_entry)
    .bind(request.account_id)
    .bind(request.transaction_id)
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
                log::info!("✅ Loan updated successfully: {}", id);
                Ok(Json(json!({
                    "success": true,
                    "message": "Loan updated successfully"
                })))
            }
        }
        Err(e) => {
            log::error!("Failed to update loan: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_loan(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<Value>, StatusCode> {
    log::info!("📥 DELETE /loans/{} - Deleting loan", id);

    let result = sqlx::query("DELETE FROM loans WHERE id = ? AND user_id = ?")
        .bind(&id)
        .bind(&auth_user.user_id)
        .execute(&pool)
        .await;

    match result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                Err(StatusCode::NOT_FOUND)
            } else {
                log::info!("✅ Loan deleted successfully: {}", id);
                Ok(Json(json!({
                    "success": true,
                    "message": "Loan deleted successfully"
                })))
            }
        }
        Err(e) => {
            log::error!("Failed to delete loan: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
