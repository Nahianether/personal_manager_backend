use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use chrono::Utc;
use sqlx::Row;

use crate::models::{Liability, CreateLiabilityRequest, UpdateLiabilityRequest};
use crate::services::DbPool;

pub async fn create_liability(
    State(pool): State<DbPool>,
    Json(request): Json<CreateLiabilityRequest>,
) -> Result<Json<Value>, StatusCode> {
    let liability = Liability::new(request);
    let due_date_str = liability.due_date.format("%Y-%m-%d %H:%M:%S").to_string();
    let created_at_str = liability.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
    let updated_at_str = liability.updated_at.format("%Y-%m-%d %H:%M:%S").to_string();
    
    let result = sqlx::query(
        "INSERT INTO liabilities (id, person_name, amount, currency, due_date, is_paid, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&liability.id)
    .bind(&liability.person_name)
    .bind(liability.amount)
    .bind(&liability.currency)
    .bind(&due_date_str)
    .bind(liability.is_paid)
    .bind(&liability.description)
    .bind(&created_at_str)
    .bind(&updated_at_str)
    .execute(&pool)
    .await;

    match result {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "data": liability
        }))),
        Err(e) => {
            log::error!("Failed to create liability: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_liabilities(
    State(pool): State<DbPool>,
) -> Result<Json<Value>, StatusCode> {
    let result = sqlx::query(
        "SELECT id, person_name, amount, currency, due_date, is_paid, description, created_at, updated_at FROM liabilities ORDER BY due_date ASC"
    )
    .fetch_all(&pool)
    .await;

    match result {
        Ok(rows) => {
            let liabilities: Vec<_> = rows.into_iter().map(|row| {
                json!({
                    "id": row.get::<String, _>("id"),
                    "person_name": row.get::<String, _>("person_name"),
                    "amount": row.get::<f64, _>("amount"),
                    "currency": row.get::<String, _>("currency"),
                    "due_date": row.get::<String, _>("due_date"),
                    "is_paid": row.get::<bool, _>("is_paid"),
                    "description": row.get::<Option<String>, _>("description"),
                    "created_at": row.get::<String, _>("created_at"),
                    "updated_at": row.get::<String, _>("updated_at")
                })
            }).collect();
            
            Ok(Json(json!({
                "success": true,
                "data": liabilities
            })))
        }
        Err(e) => {
            log::error!("Failed to get liabilities: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_liability(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
) -> Result<Json<Value>, StatusCode> {
    let result = sqlx::query(
        "SELECT id, person_name, amount, currency, due_date, is_paid, description, created_at, updated_at FROM liabilities WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await;

    match result {
        Ok(Some(row)) => {
            let liability = json!({
                "id": row.get::<String, _>("id"),
                "person_name": row.get::<String, _>("person_name"),
                "amount": row.get::<f64, _>("amount"),
                "currency": row.get::<String, _>("currency"),
                "due_date": row.get::<String, _>("due_date"),
                "is_paid": row.get::<bool, _>("is_paid"),
                "description": row.get::<Option<String>, _>("description"),
                "created_at": row.get::<String, _>("created_at"),
                "updated_at": row.get::<String, _>("updated_at")
            });
            
            Ok(Json(json!({
                "success": true,
                "data": liability
            })))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            log::error!("Failed to get liability: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_liability(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
    Json(request): Json<UpdateLiabilityRequest>,
) -> Result<Json<Value>, StatusCode> {
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let due_date_str = request.due_date.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string());
    
    let result = sqlx::query(
        "UPDATE liabilities SET person_name = COALESCE(?, person_name), amount = COALESCE(?, amount), currency = COALESCE(?, currency), due_date = COALESCE(?, due_date), is_paid = COALESCE(?, is_paid), description = COALESCE(?, description), updated_at = ? WHERE id = ?"
    )
    .bind(request.person_name)
    .bind(request.amount)
    .bind(request.currency)
    .bind(due_date_str)
    .bind(request.is_paid)
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
                    "message": "Liability updated successfully"
                })))
            }
        }
        Err(e) => {
            log::error!("Failed to update liability: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_liability(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
) -> Result<Json<Value>, StatusCode> {
    let result = sqlx::query("DELETE FROM liabilities WHERE id = ?")
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
                    "message": "Liability deleted successfully"
                })))
            }
        }
        Err(e) => {
            log::error!("Failed to delete liability: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}