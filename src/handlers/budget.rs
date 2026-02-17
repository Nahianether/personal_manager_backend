use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use chrono::Utc;
use sqlx::Row;

use crate::models::{Budget, CreateBudgetRequest, UpdateBudgetRequest};
use crate::services::DbPool;
use crate::middleware::auth::AuthUser;

pub async fn create_budget(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Json(request): Json<CreateBudgetRequest>,
) -> Result<Json<Value>, StatusCode> {
    log::info!("POST /budgets - Creating budget for user {}", auth_user.user_id);

    let budget = Budget::new(request, auth_user.user_id.clone());
    let created_at_str = budget.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
    let updated_at_str = budget.updated_at.format("%Y-%m-%d %H:%M:%S").to_string();

    let result = sqlx::query(
        "INSERT INTO budgets (id, user_id, category, amount, currency, period, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&budget.id)
    .bind(&budget.user_id)
    .bind(&budget.category)
    .bind(budget.amount)
    .bind(&budget.currency)
    .bind(&budget.period)
    .bind(&created_at_str)
    .bind(&updated_at_str)
    .execute(&pool)
    .await;

    match result {
        Ok(_) => {
            log::info!("Budget created successfully: {} ({})", budget.category, budget.id);
            Ok(Json(json!({
                "success": true,
                "data": budget
            })))
        }
        Err(e) => {
            log::error!("Failed to create budget: {}", e);
            let error_msg = e.to_string();
            if error_msg.contains("UNIQUE constraint failed: budgets.id") {
                log::warn!("Budget with ID {} already exists", budget.id);
                Err(StatusCode::CONFLICT)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

pub async fn get_budgets(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<Value>, StatusCode> {
    log::info!("GET /budgets - Fetching budgets for user {}", auth_user.user_id);

    let result = sqlx::query(
        "SELECT id, user_id, category, amount, currency, period, created_at, updated_at FROM budgets WHERE user_id = ? ORDER BY created_at DESC"
    )
    .bind(&auth_user.user_id)
    .fetch_all(&pool)
    .await;

    match result {
        Ok(rows) => {
            let budgets: Vec<_> = rows.into_iter().map(|row| {
                json!({
                    "id": row.get::<String, _>("id"),
                    "userId": row.get::<String, _>("user_id"),
                    "category": row.get::<String, _>("category"),
                    "amount": row.get::<f64, _>("amount"),
                    "currency": row.get::<String, _>("currency"),
                    "period": row.get::<String, _>("period"),
                    "createdAt": row.get::<String, _>("created_at"),
                    "updatedAt": row.get::<String, _>("updated_at")
                })
            }).collect();

            log::info!("Found {} budgets", budgets.len());
            Ok(Json(json!({
                "success": true,
                "data": budgets
            })))
        }
        Err(e) => {
            log::error!("Failed to get budgets: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_budget(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<Value>, StatusCode> {
    log::info!("GET /budgets/{} - Fetching budget by ID", id);

    let result = sqlx::query(
        "SELECT id, user_id, category, amount, currency, period, created_at, updated_at FROM budgets WHERE id = ? AND user_id = ?"
    )
    .bind(&id)
    .bind(&auth_user.user_id)
    .fetch_optional(&pool)
    .await;

    match result {
        Ok(Some(row)) => {
            let budget = json!({
                "id": row.get::<String, _>("id"),
                "userId": row.get::<String, _>("user_id"),
                "category": row.get::<String, _>("category"),
                "amount": row.get::<f64, _>("amount"),
                "currency": row.get::<String, _>("currency"),
                "period": row.get::<String, _>("period"),
                "createdAt": row.get::<String, _>("created_at"),
                "updatedAt": row.get::<String, _>("updated_at")
            });

            Ok(Json(json!({
                "success": true,
                "data": budget
            })))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            log::error!("Failed to get budget: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_budget(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Json(request): Json<UpdateBudgetRequest>,
) -> Result<Json<Value>, StatusCode> {
    log::info!("PUT /budgets/{} - Updating budget", id);

    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let result = sqlx::query(
        "UPDATE budgets SET category = COALESCE(?, category), amount = COALESCE(?, amount), currency = COALESCE(?, currency), period = COALESCE(?, period), updated_at = ? WHERE id = ? AND user_id = ?"
    )
    .bind(request.category)
    .bind(request.amount)
    .bind(request.currency)
    .bind(request.period)
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
                log::info!("Budget updated successfully: {}", id);
                Ok(Json(json!({
                    "success": true,
                    "message": "Budget updated successfully"
                })))
            }
        }
        Err(e) => {
            log::error!("Failed to update budget: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_budget(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<Value>, StatusCode> {
    log::info!("DELETE /budgets/{} - Deleting budget", id);

    let result = sqlx::query("DELETE FROM budgets WHERE id = ? AND user_id = ?")
        .bind(&id)
        .bind(&auth_user.user_id)
        .execute(&pool)
        .await;

    match result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                Err(StatusCode::NOT_FOUND)
            } else {
                log::info!("Budget deleted successfully: {}", id);
                Ok(Json(json!({
                    "success": true,
                    "message": "Budget deleted successfully"
                })))
            }
        }
        Err(e) => {
            log::error!("Failed to delete budget: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
