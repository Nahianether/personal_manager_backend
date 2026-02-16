use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use sqlx::Row;

use crate::services::DbPool;
use crate::middleware::auth::AuthUser;

pub async fn get_preferences(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<Value>, StatusCode> {
    log::info!("GET /api/preferences - Fetching preferences for user {}", auth_user.user_id);

    let result = sqlx::query(
        "SELECT user_id, display_currency, updated_at FROM user_preferences WHERE user_id = ?"
    )
    .bind(&auth_user.user_id)
    .fetch_optional(&pool)
    .await;

    match result {
        Ok(Some(row)) => {
            Ok(Json(json!({
                "success": true,
                "data": {
                    "displayCurrency": row.get::<String, _>("display_currency"),
                    "updatedAt": row.get::<String, _>("updated_at")
                }
            })))
        }
        Ok(None) => {
            // Return defaults if no preferences saved yet
            Ok(Json(json!({
                "success": true,
                "data": {
                    "displayCurrency": "BDT",
                    "updatedAt": null
                }
            })))
        }
        Err(e) => {
            log::error!("Failed to get preferences: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_preferences(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<Value>, StatusCode> {
    log::info!("PUT /api/preferences - Updating preferences for user {}", auth_user.user_id);

    let display_currency = request.get("display_currency")
        .or_else(|| request.get("displayCurrency"))
        .and_then(|v| v.as_str())
        .unwrap_or("BDT");

    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let result = sqlx::query(
        "INSERT INTO user_preferences (user_id, display_currency, updated_at) VALUES (?, ?, ?) ON CONFLICT(user_id) DO UPDATE SET display_currency = ?, updated_at = ?"
    )
    .bind(&auth_user.user_id)
    .bind(display_currency)
    .bind(&now)
    .bind(display_currency)
    .bind(&now)
    .execute(&pool)
    .await;

    match result {
        Ok(_) => {
            log::info!("Preferences updated: display_currency={}", display_currency);
            Ok(Json(json!({
                "success": true,
                "data": {
                    "displayCurrency": display_currency,
                    "updatedAt": now
                }
            })))
        }
        Err(e) => {
            log::error!("Failed to update preferences: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
