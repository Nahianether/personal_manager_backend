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

pub async fn create_loan(
    State(_pool): State<DbPool>,
    Json(_request): Json<CreateLoanRequest>,
) -> Result<Json<Value>, StatusCode> {
    // TODO: Implement CRUD handlers with authentication
    // This handler is temporarily disabled and needs to be updated to use authentication
    
    Err(StatusCode::NOT_IMPLEMENTED)
}

pub async fn get_loans(
    State(pool): State<DbPool>,
) -> Result<Json<Value>, StatusCode> {
    let result = sqlx::query(
        "SELECT id, person_name, amount, currency, loan_date, return_date, is_returned, description, created_at, updated_at FROM loans ORDER BY loan_date DESC"
    )
    .fetch_all(&pool)
    .await;

    match result {
        Ok(rows) => {
            let loans: Vec<_> = rows.into_iter().map(|row| {
                json!({
                    "id": row.get::<String, _>("id"),
                    "person_name": row.get::<String, _>("person_name"),
                    "amount": row.get::<f64, _>("amount"),
                    "currency": row.get::<String, _>("currency"),
                    "loan_date": row.get::<String, _>("loan_date"),
                    "return_date": row.get::<Option<String>, _>("return_date"),
                    "is_returned": row.get::<bool, _>("is_returned"),
                    "description": row.get::<Option<String>, _>("description"),
                    "created_at": row.get::<String, _>("created_at"),
                    "updated_at": row.get::<String, _>("updated_at")
                })
            }).collect();
            
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
) -> Result<Json<Value>, StatusCode> {
    let result = sqlx::query(
        "SELECT id, person_name, amount, currency, loan_date, return_date, is_returned, description, created_at, updated_at FROM loans WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await;

    match result {
        Ok(Some(row)) => {
            let loan = json!({
                "id": row.get::<String, _>("id"),
                "person_name": row.get::<String, _>("person_name"),
                "amount": row.get::<f64, _>("amount"),
                "currency": row.get::<String, _>("currency"),
                "loan_date": row.get::<String, _>("loan_date"),
                "return_date": row.get::<Option<String>, _>("return_date"),
                "is_returned": row.get::<bool, _>("is_returned"),
                "description": row.get::<Option<String>, _>("description"),
                "created_at": row.get::<String, _>("created_at"),
                "updated_at": row.get::<String, _>("updated_at")
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
    Json(request): Json<UpdateLoanRequest>,
) -> Result<Json<Value>, StatusCode> {
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let loan_date_str = request.loan_date.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string());
    let return_date_str = request.return_date.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string());
    
    let result = sqlx::query(
        "UPDATE loans SET person_name = COALESCE(?, person_name), amount = COALESCE(?, amount), currency = COALESCE(?, currency), loan_date = COALESCE(?, loan_date), return_date = COALESCE(?, return_date), is_returned = COALESCE(?, is_returned), description = COALESCE(?, description), updated_at = ? WHERE id = ?"
    )
    .bind(request.person_name)
    .bind(request.amount)
    .bind(request.currency)
    .bind(loan_date_str)
    .bind(return_date_str)
    .bind(request.is_returned)
    .bind(request.description)
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
) -> Result<Json<Value>, StatusCode> {
    let result = sqlx::query("DELETE FROM loans WHERE id = ?")
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