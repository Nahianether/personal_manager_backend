use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RecurringTransaction {
    pub id: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "accountId")]
    pub account_id: String,
    #[serde(rename = "transactionType")]
    pub transaction_type: String,
    pub amount: f64,
    pub currency: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub frequency: String,
    #[serde(rename = "startDate")]
    pub start_date: DateTime<Utc>,
    #[serde(rename = "endDate")]
    pub end_date: Option<DateTime<Utc>>,
    #[serde(rename = "nextDueDate")]
    pub next_due_date: DateTime<Utc>,
    #[serde(rename = "isActive")]
    pub is_active: bool,
    #[serde(rename = "savingsGoalId")]
    pub savings_goal_id: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRecurringTransactionRequest {
    pub id: Option<String>,
    pub account_id: String,
    pub transaction_type: String,
    pub amount: f64,
    pub currency: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub frequency: Option<String>,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub next_due_date: DateTime<Utc>,
    pub is_active: Option<bool>,
    pub savings_goal_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRecurringTransactionRequest {
    pub account_id: Option<String>,
    pub transaction_type: Option<String>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub frequency: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub next_due_date: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
    pub savings_goal_id: Option<String>,
}

impl RecurringTransaction {
    pub fn new(request: CreateRecurringTransactionRequest, user_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: request.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            user_id,
            account_id: request.account_id,
            transaction_type: request.transaction_type,
            amount: request.amount,
            currency: request.currency.unwrap_or_else(|| "BDT".to_string()),
            category: request.category,
            description: request.description,
            frequency: request.frequency.unwrap_or_else(|| "monthly".to_string()),
            start_date: request.start_date,
            end_date: request.end_date,
            next_due_date: request.next_due_date,
            is_active: request.is_active.unwrap_or(true),
            savings_goal_id: request.savings_goal_id,
            created_at: now,
            updated_at: now,
        }
    }
}
