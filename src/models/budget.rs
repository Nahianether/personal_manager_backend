use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Budget {
    pub id: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub category: String,
    pub amount: f64,
    pub currency: String,
    pub period: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBudgetRequest {
    pub id: Option<String>,
    pub category: String,
    pub amount: f64,
    pub currency: Option<String>,
    pub period: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBudgetRequest {
    pub category: Option<String>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub period: Option<String>,
}

impl Budget {
    pub fn new(request: CreateBudgetRequest, user_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: request.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            user_id,
            category: request.category,
            amount: request.amount,
            currency: request.currency.unwrap_or_else(|| "BDT".to_string()),
            period: request.period.unwrap_or_else(|| "monthly".to_string()),
            created_at: now,
            updated_at: now,
        }
    }
}
