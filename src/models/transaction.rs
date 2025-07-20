use serde::{Deserialize, Serialize, Deserializer};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDateTime};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Transaction {
    pub id: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "accountId")]
    pub account_id: String,
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    pub amount: f64,
    pub currency: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub date: DateTime<Utc>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
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
    #[serde(alias = "accountId")]
    pub account_id: String,
    #[serde(alias = "type")]
    pub transaction_type: TransactionType,
    pub amount: f64,
    pub currency: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_datetime")]
    pub date: Option<DateTime<Utc>>,
}

fn deserialize_optional_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) => {
            if s.is_empty() {
                return Ok(None);
            }
            
            // Try different formats
            // ISO format with Z
            if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
                return Ok(Some(dt.with_timezone(&Utc)));
            }
            
            // Try milliseconds timestamp
            if let Ok(timestamp) = s.parse::<i64>() {
                if let Some(dt) = DateTime::from_timestamp_millis(timestamp) {
                    return Ok(Some(dt));
                }
            }
            
            // Try seconds timestamp
            if let Ok(timestamp) = s.parse::<i64>() {
                if let Some(dt) = DateTime::from_timestamp(timestamp, 0) {
                    return Ok(Some(dt));
                }
            }
            
            // Try parsing as naive datetime with microseconds (Flutter format)
            if let Ok(naive) = NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f") {
                return Ok(Some(naive.and_utc()));
            }
            
            // Try parsing as naive datetime and assume UTC
            if let Ok(naive) = NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S") {
                return Ok(Some(naive.and_utc()));
            }
            
            // Try parsing as date only
            if let Ok(naive) = NaiveDateTime::parse_from_str(&format!("{} 00:00:00", s), "%Y-%m-%d %H:%M:%S") {
                return Ok(Some(naive.and_utc()));
            }
            
            Err(serde::de::Error::custom(format!("Unable to parse date: {}", s)))
        }
        None => Ok(None),
    }
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
    pub fn new(request: CreateTransactionRequest, user_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
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