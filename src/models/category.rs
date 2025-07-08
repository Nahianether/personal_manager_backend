use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub category_type: CategoryType,
    pub icon: String,
    pub color: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum CategoryType {
    #[sqlx(rename = "income")]
    Income,
    #[sqlx(rename = "expense")]
    Expense,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub category_type: CategoryType,
    pub icon: String,
    pub color: String,
    pub is_default: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub category_type: Option<CategoryType>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_default: Option<bool>,
}

impl Category {
    pub fn new(request: CreateCategoryRequest) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: request.name,
            category_type: request.category_type,
            icon: request.icon,
            color: request.color,
            is_default: request.is_default.unwrap_or(false),
            created_at: Utc::now(),
        }
    }
}

pub struct DefaultCategories;

impl DefaultCategories {
    pub fn get_income_categories() -> Vec<Category> {
        vec![
            Category {
                id: Uuid::new_v4().to_string(),
                name: "Salary".to_string(),
                category_type: CategoryType::Income,
                icon: "💰".to_string(),
                color: "#4CAF50".to_string(),
                is_default: true,
                created_at: Utc::now(),
            },
            Category {
                id: Uuid::new_v4().to_string(),
                name: "Business".to_string(),
                category_type: CategoryType::Income,
                icon: "💼".to_string(),
                color: "#2196F3".to_string(),
                is_default: true,
                created_at: Utc::now(),
            },
            Category {
                id: Uuid::new_v4().to_string(),
                name: "Investment".to_string(),
                category_type: CategoryType::Income,
                icon: "📈".to_string(),
                color: "#FF9800".to_string(),
                is_default: true,
                created_at: Utc::now(),
            },
            Category {
                id: Uuid::new_v4().to_string(),
                name: "Gift".to_string(),
                category_type: CategoryType::Income,
                icon: "🎁".to_string(),
                color: "#E91E63".to_string(),
                is_default: true,
                created_at: Utc::now(),
            },
        ]
    }

    pub fn get_expense_categories() -> Vec<Category> {
        vec![
            Category {
                id: Uuid::new_v4().to_string(),
                name: "Food".to_string(),
                category_type: CategoryType::Expense,
                icon: "🍔".to_string(),
                color: "#FF5722".to_string(),
                is_default: true,
                created_at: Utc::now(),
            },
            Category {
                id: Uuid::new_v4().to_string(),
                name: "Transportation".to_string(),
                category_type: CategoryType::Expense,
                icon: "🚗".to_string(),
                color: "#607D8B".to_string(),
                is_default: true,
                created_at: Utc::now(),
            },
            Category {
                id: Uuid::new_v4().to_string(),
                name: "Shopping".to_string(),
                category_type: CategoryType::Expense,
                icon: "🛍️".to_string(),
                color: "#9C27B0".to_string(),
                is_default: true,
                created_at: Utc::now(),
            },
            Category {
                id: Uuid::new_v4().to_string(),
                name: "Entertainment".to_string(),
                category_type: CategoryType::Expense,
                icon: "🎬".to_string(),
                color: "#673AB7".to_string(),
                is_default: true,
                created_at: Utc::now(),
            },
            Category {
                id: Uuid::new_v4().to_string(),
                name: "Bills".to_string(),
                category_type: CategoryType::Expense,
                icon: "💡".to_string(),
                color: "#795548".to_string(),
                is_default: true,
                created_at: Utc::now(),
            },
            Category {
                id: Uuid::new_v4().to_string(),
                name: "Medical".to_string(),
                category_type: CategoryType::Expense,
                icon: "⚕️".to_string(),
                color: "#F44336".to_string(),
                is_default: true,
                created_at: Utc::now(),
            },
        ]
    }

    pub fn get_all_default_categories() -> Vec<Category> {
        let mut categories = Self::get_income_categories();
        categories.extend(Self::get_expense_categories());
        categories
    }
}