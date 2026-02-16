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
    #[serde(rename = "isHistoricalEntry")]
    pub is_historical_entry: bool,
    #[serde(rename = "accountId")]
    pub account_id: Option<String>,
    #[serde(rename = "transactionId")]
    pub transaction_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLiabilityRequest {
    pub id: Option<String>,
    pub person_name: String,
    pub amount: f64,
    pub currency: Option<String>,
    pub due_date: DateTime<Utc>,
    pub is_paid: Option<bool>,
    pub description: Option<String>,
    pub is_historical_entry: Option<bool>,
    pub account_id: Option<String>,
    pub transaction_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLiabilityRequest {
    pub person_name: Option<String>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub is_paid: Option<bool>,
    pub description: Option<String>,
    pub is_historical_entry: Option<bool>,
    pub account_id: Option<String>,
    pub transaction_id: Option<String>,
}

impl Liability {
    pub fn new(request: CreateLiabilityRequest, user_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: request.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            user_id,
            person_name: request.person_name,
            amount: request.amount,
            currency: request.currency.unwrap_or_else(|| "BDT".to_string()),
            due_date: request.due_date,
            is_paid: request.is_paid.unwrap_or(false),
            description: request.description,
            created_at: now,
            updated_at: now,
            is_historical_entry: request.is_historical_entry.unwrap_or(false),
            account_id: request.account_id,
            transaction_id: request.transaction_id,
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
