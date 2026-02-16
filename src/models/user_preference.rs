use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserPreference {
    pub user_id: String,
    #[serde(rename = "displayCurrency")]
    pub display_currency: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePreferenceRequest {
    pub display_currency: Option<String>,
}
