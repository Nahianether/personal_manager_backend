use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use chrono::Utc;
use sqlx::Row;

use crate::models::{SavingsGoal, CreateSavingsGoalRequest, UpdateSavingsGoalRequest};
use crate::services::DbPool;
use crate::middleware::auth::AuthUser;

pub async fn create_savings_goal(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Json(request): Json<CreateSavingsGoalRequest>,
) -> Result<Json<Value>, StatusCode> {
    log::info!("POST /savings-goals - Creating savings goal for user {}", auth_user.user_id);

    let goal = SavingsGoal::new(request, auth_user.user_id.clone());
    let target_date_str = goal.target_date.format("%Y-%m-%d %H:%M:%S").to_string();
    let created_at_str = goal.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
    let updated_at_str = goal.updated_at.format("%Y-%m-%d %H:%M:%S").to_string();

    let result = sqlx::query(
        "INSERT INTO savings_goals (id, user_id, name, target_amount, current_amount, currency, target_date, description, account_id, priority, is_completed, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&goal.id)
    .bind(&goal.user_id)
    .bind(&goal.name)
    .bind(goal.target_amount)
    .bind(goal.current_amount)
    .bind(&goal.currency)
    .bind(&target_date_str)
    .bind(&goal.description)
    .bind(&goal.account_id)
    .bind(&goal.priority)
    .bind(goal.is_completed)
    .bind(&created_at_str)
    .bind(&updated_at_str)
    .execute(&pool)
    .await;

    match result {
        Ok(_) => {
            log::info!("Savings goal created successfully: {} ({})", goal.name, goal.id);
            Ok(Json(json!({
                "success": true,
                "data": goal
            })))
        }
        Err(e) => {
            log::error!("Failed to create savings goal: {}", e);
            let error_msg = e.to_string();
            if error_msg.contains("UNIQUE constraint failed: savings_goals.id") {
                log::warn!("Savings goal with ID {} already exists", goal.id);
                Err(StatusCode::CONFLICT)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

pub async fn get_savings_goals(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<Value>, StatusCode> {
    log::info!("GET /savings-goals - Fetching savings goals for user {}", auth_user.user_id);

    let result = sqlx::query(
        "SELECT id, user_id, name, target_amount, current_amount, currency, target_date, description, account_id, priority, is_completed, created_at, updated_at FROM savings_goals WHERE user_id = ? ORDER BY target_date ASC"
    )
    .bind(&auth_user.user_id)
    .fetch_all(&pool)
    .await;

    match result {
        Ok(rows) => {
            let goals: Vec<_> = rows.into_iter().map(|row| {
                json!({
                    "id": row.get::<String, _>("id"),
                    "user_id": row.get::<String, _>("user_id"),
                    "name": row.get::<String, _>("name"),
                    "target_amount": row.get::<f64, _>("target_amount"),
                    "current_amount": row.get::<f64, _>("current_amount"),
                    "currency": row.get::<String, _>("currency"),
                    "target_date": row.get::<String, _>("target_date"),
                    "description": row.get::<Option<String>, _>("description"),
                    "account_id": row.get::<Option<String>, _>("account_id"),
                    "priority": row.get::<String, _>("priority"),
                    "is_completed": row.get::<bool, _>("is_completed"),
                    "created_at": row.get::<String, _>("created_at"),
                    "updated_at": row.get::<String, _>("updated_at")
                })
            }).collect();

            log::info!("Found {} savings goals", goals.len());
            Ok(Json(json!({
                "success": true,
                "data": goals
            })))
        }
        Err(e) => {
            log::error!("Failed to get savings goals: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_savings_goal(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<Value>, StatusCode> {
    log::info!("GET /savings-goals/{} - Fetching savings goal by ID", id);

    let result = sqlx::query(
        "SELECT id, user_id, name, target_amount, current_amount, currency, target_date, description, account_id, priority, is_completed, created_at, updated_at FROM savings_goals WHERE id = ? AND user_id = ?"
    )
    .bind(&id)
    .bind(&auth_user.user_id)
    .fetch_optional(&pool)
    .await;

    match result {
        Ok(Some(row)) => {
            let goal = json!({
                "id": row.get::<String, _>("id"),
                "user_id": row.get::<String, _>("user_id"),
                "name": row.get::<String, _>("name"),
                "target_amount": row.get::<f64, _>("target_amount"),
                "current_amount": row.get::<f64, _>("current_amount"),
                "currency": row.get::<String, _>("currency"),
                "target_date": row.get::<String, _>("target_date"),
                "description": row.get::<Option<String>, _>("description"),
                "account_id": row.get::<Option<String>, _>("account_id"),
                "priority": row.get::<String, _>("priority"),
                "is_completed": row.get::<bool, _>("is_completed"),
                "created_at": row.get::<String, _>("created_at"),
                "updated_at": row.get::<String, _>("updated_at")
            });

            Ok(Json(json!({
                "success": true,
                "data": goal
            })))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            log::error!("Failed to get savings goal: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_savings_goal(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Json(request): Json<UpdateSavingsGoalRequest>,
) -> Result<Json<Value>, StatusCode> {
    log::info!("PUT /savings-goals/{} - Updating savings goal", id);

    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let target_date_str = request.target_date.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string());

    let result = sqlx::query(
        "UPDATE savings_goals SET name = COALESCE(?, name), target_amount = COALESCE(?, target_amount), current_amount = COALESCE(?, current_amount), currency = COALESCE(?, currency), target_date = COALESCE(?, target_date), description = COALESCE(?, description), account_id = COALESCE(?, account_id), priority = COALESCE(?, priority), is_completed = COALESCE(?, is_completed), updated_at = ? WHERE id = ? AND user_id = ?"
    )
    .bind(request.name)
    .bind(request.target_amount)
    .bind(request.current_amount)
    .bind(request.currency)
    .bind(target_date_str)
    .bind(request.description)
    .bind(request.account_id)
    .bind(request.priority)
    .bind(request.is_completed)
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
                log::info!("Savings goal updated successfully: {}", id);
                Ok(Json(json!({
                    "success": true,
                    "message": "Savings goal updated successfully"
                })))
            }
        }
        Err(e) => {
            log::error!("Failed to update savings goal: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_savings_goal(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<Value>, StatusCode> {
    log::info!("DELETE /savings-goals/{} - Deleting savings goal", id);

    let result = sqlx::query("DELETE FROM savings_goals WHERE id = ? AND user_id = ?")
        .bind(&id)
        .bind(&auth_user.user_id)
        .execute(&pool)
        .await;

    match result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                Err(StatusCode::NOT_FOUND)
            } else {
                log::info!("Savings goal deleted successfully: {}", id);
                Ok(Json(json!({
                    "success": true,
                    "message": "Savings goal deleted successfully"
                })))
            }
        }
        Err(e) => {
            log::error!("Failed to delete savings goal: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
