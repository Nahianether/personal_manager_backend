use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Transaction {
    pub id: String,
    pub account_id: String,
    pub transaction_type: TransactionType,
    pub amount: f64,
    pub currency: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum TransactionType {
    #[sqlx(rename = "income")]
    Income,
    #[sqlx(rename = "expense")]
    Expense,
    #[sqlx(rename = "transfer")]
    Transfer,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub account_id: String,
    pub transaction_type: TransactionType,
    pub amount: f64,
    pub currency: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub date: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTransactionRequest {
    pub account_id: Option<String>,
    pub transaction_type: Option<TransactionType>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub date: Option<DateTime<Utc>>,
}

impl Transaction {
    pub fn new(request: CreateTransactionRequest) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            account_id: request.account_id,
            transaction_type: request.transaction_type,
            amount: request.amount,
            currency: request.currency.unwrap_or_else(|| "BDT".to_string()),
            category: request.category,
            description: request.description,
            date: request.date.unwrap_or(now),
            created_at: now,
        }
    }
}

pub struct TransactionCategories;

impl TransactionCategories {
    pub fn get_income_categories() -> Vec<&'static str> {
        vec![
            "Salary",
            "Business",
            "Investment",
            "Gift",
            "Other Income",
        ]
    }

    pub fn get_expense_categories() -> Vec<&'static str> {
        vec![
            "Food",
            "Transportation",
            "Shopping",
            "Entertainment",
            "Bills",
            "Medical",
            "Education",
            "Other Expense",
        ]
    }
}