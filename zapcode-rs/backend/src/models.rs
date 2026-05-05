use crate::db::schema::*;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = merchants)]
#[serde(rename_all = "camelCase")]
pub struct Merchant {
    pub id: String,
    pub email: String,
    pub business_name: String,
    pub wallet_id: String,
    pub wallet_address: String,
    pub public_key: String,
    pub currency: String,
    pub country: String,
    pub network: String,
    pub logo_url: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicMerchant {
    pub id: String,
    pub business_name: String,
    pub wallet_address: String,
    pub currency: String,
    pub network: String,
    pub logo_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = transactions)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub id: String,
    pub merchant_id: String,
    pub tx_hash: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: bigdecimal::BigDecimal,
    pub currency: String,
    pub status: String,
    pub block_number: Option<String>,
    pub note: Option<String>,
    pub email_sent: bool,
    pub detected_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = buyers)]
#[serde(rename_all = "camelCase")]
pub struct Buyer {
    pub id: String,
    pub wallet_id: String,
    pub wallet_address: String,
    pub public_key: String,
    pub network: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingData {
    pub business_name: String,
    pub currency: String,
    pub country: String,
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub total_revenue: f64,
    pub order_count: i64,
    pub avg_order_value: f64,
    pub today_revenue: f64,
    pub today_orders: i64,
    pub week_revenue: f64,
    pub week_orders: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicStats {
    pub merchants: i64,
    pub payments: i64,
    pub unique_senders: i64,
}
