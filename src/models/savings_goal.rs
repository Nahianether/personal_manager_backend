use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SavingsGoal {
    pub id: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub name: String,
    #[serde(rename = "targetAmount")]
    pub target_amount: f64,
    #[serde(rename = "currentAmount")]
    pub current_amount: f64,
    pub currency: String,
    #[serde(rename = "targetDate")]
    pub target_date: DateTime<Utc>,
    pub description: Option<String>,
    #[serde(rename = "accountId")]
    pub account_id: Option<String>,
    pub priority: String,
    #[serde(rename = "isCompleted")]
    pub is_completed: bool,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSavingsGoalRequest {
    pub id: Option<String>,
    pub name: String,
    pub target_amount: f64,
    pub currency: Option<String>,
    pub target_date: DateTime<Utc>,
    pub description: Option<String>,
    pub account_id: Option<String>,
    pub priority: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSavingsGoalRequest {
    pub name: Option<String>,
    pub target_amount: Option<f64>,
    pub current_amount: Option<f64>,
    pub currency: Option<String>,
    pub target_date: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub account_id: Option<String>,
    pub priority: Option<String>,
    pub is_completed: Option<bool>,
}

impl SavingsGoal {
    pub fn new(request: CreateSavingsGoalRequest, user_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: request.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            user_id,
            name: request.name,
            target_amount: request.target_amount,
            current_amount: 0.0,
            currency: request.currency.unwrap_or_else(|| "BDT".to_string()),
            target_date: request.target_date,
            description: request.description,
            account_id: request.account_id,
            priority: request.priority.unwrap_or_else(|| "medium".to_string()),
            is_completed: false,
            created_at: now,
            updated_at: now,
        }
    }
}
