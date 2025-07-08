use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use sqlx::Row;

use crate::models::{Category, CreateCategoryRequest, UpdateCategoryRequest};
use crate::services::DbPool;

pub async fn create_category(
    State(pool): State<DbPool>,
    Json(request): Json<CreateCategoryRequest>,
) -> Result<Json<Value>, StatusCode> {
    let category = Category::new(request);
    let category_type_str = format!("{:?}", category.category_type).to_lowercase();
    let created_at_str = category.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
    
    let result = sqlx::query(
        "INSERT INTO categories (id, name, category_type, icon, color, is_default, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&category.id)
    .bind(&category.name)
    .bind(&category_type_str)
    .bind(&category.icon)
    .bind(&category.color)
    .bind(category.is_default)
    .bind(&created_at_str)
    .execute(&pool)
    .await;

    match result {
        Ok(_) => Ok(Json(json!({
            "success": true,
            "data": category
        }))),
        Err(e) => {
            log::error!("Failed to create category: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_categories(
    State(pool): State<DbPool>,
) -> Result<Json<Value>, StatusCode> {
    let result = sqlx::query(
        "SELECT id, name, category_type, icon, color, is_default, created_at FROM categories ORDER BY is_default DESC, created_at ASC"
    )
    .fetch_all(&pool)
    .await;

    match result {
        Ok(rows) => {
            let categories: Vec<_> = rows.into_iter().map(|row| {
                json!({
                    "id": row.get::<String, _>("id"),
                    "name": row.get::<String, _>("name"),
                    "category_type": row.get::<String, _>("category_type"),
                    "icon": row.get::<String, _>("icon"),
                    "color": row.get::<String, _>("color"),
                    "is_default": row.get::<bool, _>("is_default"),
                    "created_at": row.get::<String, _>("created_at")
                })
            }).collect();
            
            Ok(Json(json!({
                "success": true,
                "data": categories
            })))
        }
        Err(e) => {
            log::error!("Failed to get categories: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_category(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
) -> Result<Json<Value>, StatusCode> {
    let result = sqlx::query(
        "SELECT id, name, category_type, icon, color, is_default, created_at FROM categories WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await;

    match result {
        Ok(Some(row)) => {
            let category = json!({
                "id": row.get::<String, _>("id"),
                "name": row.get::<String, _>("name"),
                "category_type": row.get::<String, _>("category_type"),
                "icon": row.get::<String, _>("icon"),
                "color": row.get::<String, _>("color"),
                "is_default": row.get::<bool, _>("is_default"),
                "created_at": row.get::<String, _>("created_at")
            });
            
            Ok(Json(json!({
                "success": true,
                "data": category
            })))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            log::error!("Failed to get category: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_category(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
    Json(request): Json<UpdateCategoryRequest>,
) -> Result<Json<Value>, StatusCode> {
    let category_type_str = request.category_type.map(|t| format!("{:?}", t).to_lowercase());
    
    let result = sqlx::query(
        "UPDATE categories SET name = COALESCE(?, name), category_type = COALESCE(?, category_type), icon = COALESCE(?, icon), color = COALESCE(?, color), is_default = COALESCE(?, is_default) WHERE id = ?"
    )
    .bind(request.name)
    .bind(category_type_str)
    .bind(request.icon)
    .bind(request.color)
    .bind(request.is_default)
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
                    "message": "Category updated successfully"
                })))
            }
        }
        Err(e) => {
            log::error!("Failed to update category: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_category(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
) -> Result<Json<Value>, StatusCode> {
    let result = sqlx::query("DELETE FROM categories WHERE id = ?")
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
                    "message": "Category deleted successfully"
                })))
            }
        }
        Err(e) => {
            log::error!("Failed to delete category: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}