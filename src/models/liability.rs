use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Liability {
    pub id: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "personName")]
    pub person_name: String,
    pub amount: f64,
    pub currency: String,
    #[serde(rename = "dueDate")]
    pub due_date: DateTime<Utc>,
    #[serde(rename = "isPaid")]
    pub is_paid: bool,
    pub description: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLiabilityRequest {
    pub person_name: String,
    pub amount: f64,
    pub currency: Option<String>,
    pub due_date: DateTime<Utc>,
    pub is_paid: Option<bool>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLiabilityRequest {
    pub person_name: Option<String>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub is_paid: Option<bool>,
    pub description: Option<String>,
}

impl Liability {
    pub fn new(request: CreateLiabilityRequest, user_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            person_name: request.person_name,
            amount: request.amount,
            currency: request.currency.unwrap_or_else(|| "BDT".to_string()),
            due_date: request.due_date,
            is_paid: request.is_paid.unwrap_or(false),
            description: request.description,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_overdue(&self) -> bool {
        !self.is_paid && self.due_date < Utc::now()
    }

    pub fn days_until_due(&self) -> i64 {
        if self.is_paid {
            0
        } else {
            (self.due_date - Utc::now()).num_days()
        }
    }
}