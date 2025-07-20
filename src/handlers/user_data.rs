use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use crate::models::{Account, Transaction, Loan, Liability};
use crate::services::database::DbPool;
use crate::middleware::AuthUser;

pub async fn get_user_accounts(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let accounts = sqlx::query_as::<_, Account>(
        "SELECT * FROM accounts WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(&auth_user.user_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to fetch accounts"
            })),
        )
    })?;

    Ok(Json(json!({
        "accounts": accounts
    })))
}

pub async fn get_user_transactions(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let transactions = sqlx::query_as::<_, Transaction>(
        "SELECT * FROM transactions WHERE user_id = ? ORDER BY date DESC",
    )
    .bind(&auth_user.user_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to fetch transactions"
            })),
        )
    })?;

    Ok(Json(json!({
        "transactions": transactions
    })))
}

pub async fn get_user_loans(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let loans = sqlx::query_as::<_, Loan>(
        "SELECT * FROM loans WHERE user_id = ? ORDER BY loan_date DESC",
    )
    .bind(&auth_user.user_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to fetch loans"
            })),
        )
    })?;

    Ok(Json(json!({
        "loans": loans
    })))
}

pub async fn get_user_liabilities(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let liabilities = sqlx::query_as::<_, Liability>(
        "SELECT * FROM liabilities WHERE user_id = ? ORDER BY due_date ASC",
    )
    .bind(&auth_user.user_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to fetch liabilities"
            })),
        )
    })?;

    Ok(Json(json!({
        "liabilities": liabilities
    })))
}