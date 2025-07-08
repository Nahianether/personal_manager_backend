use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub account_type: AccountType,
    pub balance: f64,
    pub currency: String,
    pub credit_limit: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum AccountType {
    #[sqlx(rename = "wallet")]
    Wallet,
    #[sqlx(rename = "bank")]
    Bank,
    #[sqlx(rename = "mobile_banking")]
    MobileBanking,
    #[sqlx(rename = "cash")]
    Cash,
    #[sqlx(rename = "investment")]
    Investment,
    #[sqlx(rename = "savings")]
    Savings,
    #[sqlx(rename = "credit_card")]
    CreditCard,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub name: String,
    pub account_type: AccountType,
    pub balance: f64,
    pub currency: Option<String>,
    pub credit_limit: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAccountRequest {
    pub name: Option<String>,
    pub account_type: Option<AccountType>,
    pub balance: Option<f64>,
    pub currency: Option<String>,
    pub credit_limit: Option<f64>,
}

impl Account {
    pub fn new(request: CreateAccountRequest) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: request.name,
            account_type: request.account_type,
            balance: request.balance,
            currency: request.currency.unwrap_or_else(|| "BDT".to_string()),
            credit_limit: request.credit_limit,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn available_credit(&self) -> f64 {
        match self.account_type {
            AccountType::CreditCard => {
                if let Some(limit) = self.credit_limit {
                    limit + self.balance
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }

    pub fn used_amount(&self) -> f64 {
        match self.account_type {
            AccountType::CreditCard => -self.balance,
            _ => 0.0,
        }
    }

    pub fn display_balance(&self) -> f64 {
        match self.account_type {
            AccountType::CreditCard => self.available_credit(),
            _ => self.balance,
        }
    }

    pub fn is_credit_card(&self) -> bool {
        matches!(self.account_type, AccountType::CreditCard)
    }
}