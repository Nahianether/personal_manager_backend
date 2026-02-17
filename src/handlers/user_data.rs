use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use sqlx::Row;
use crate::models::{Account, Transaction, Loan, Liability, Budget, RecurringTransaction};
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

pub async fn get_user_budgets(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let budgets = sqlx::query_as::<_, Budget>(
        "SELECT * FROM budgets WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(&auth_user.user_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to fetch budgets"
            })),
        )
    })?;

    Ok(Json(json!({
        "budgets": budgets
    })))
}

pub async fn get_user_savings_goals(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let rows = sqlx::query(
        "SELECT id, user_id, name, target_amount, current_amount, currency, target_date, description, account_id, priority, is_completed, created_at, updated_at FROM savings_goals WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(&auth_user.user_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to fetch savings goals"
            })),
        )
    })?;

    let savings_goals: Vec<_> = rows.into_iter().map(|row| {
        json!({
            "id": row.get::<String, _>("id"),
            "userId": row.get::<String, _>("user_id"),
            "name": row.get::<String, _>("name"),
            "targetAmount": row.get::<f64, _>("target_amount"),
            "currentAmount": row.get::<f64, _>("current_amount"),
            "currency": row.get::<String, _>("currency"),
            "targetDate": row.get::<String, _>("target_date"),
            "description": row.get::<Option<String>, _>("description"),
            "accountId": row.get::<Option<String>, _>("account_id"),
            "priority": row.get::<String, _>("priority"),
            "isCompleted": row.get::<bool, _>("is_completed"),
            "createdAt": row.get::<String, _>("created_at"),
            "updatedAt": row.get::<String, _>("updated_at")
        })
    }).collect();

    Ok(Json(json!({
        "savings_goals": savings_goals
    })))
}

pub async fn get_user_categories(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let rows = sqlx::query(
        "SELECT id, name, category_type, icon, color, is_default, created_at, user_id, updated_at FROM categories WHERE user_id = ? OR user_id = '' ORDER BY created_at DESC",
    )
    .bind(&auth_user.user_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to fetch categories"
            })),
        )
    })?;

    let categories: Vec<_> = rows.into_iter().map(|row| {
        json!({
            "id": row.get::<String, _>("id"),
            "name": row.get::<String, _>("name"),
            "categoryType": row.get::<String, _>("category_type"),
            "icon": row.get::<String, _>("icon"),
            "color": row.get::<String, _>("color"),
            "isDefault": row.get::<bool, _>("is_default"),
            "createdAt": row.get::<String, _>("created_at"),
            "userId": row.get::<String, _>("user_id"),
            "updatedAt": row.get::<Option<String>, _>("updated_at")
        })
    }).collect();

    Ok(Json(json!({
        "categories": categories
    })))
}

pub async fn get_user_recurring_transactions(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let recurring_transactions = sqlx::query_as::<_, RecurringTransaction>(
        "SELECT * FROM recurring_transactions WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(&auth_user.user_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to fetch recurring transactions"
            })),
        )
    })?;

    Ok(Json(json!({
        "recurring_transactions": recurring_transactions
    })))
}