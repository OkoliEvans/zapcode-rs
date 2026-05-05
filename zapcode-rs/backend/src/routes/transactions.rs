use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use bigdecimal::ToPrimitive;
use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    db::{
        schema::{merchants, transactions},
        DbPool,
    },
    models::{Merchant, Stats, Transaction},
    utils::auth::AuthUser,
};

#[derive(Deserialize)]
struct ListQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LatestResponse {
    latest_id: Option<String>,
    latest_at: Option<chrono::DateTime<Utc>>,
}

pub fn router() -> Router<DbPool> {
    Router::new()
        .route("/", get(get_transactions))
        .route("/stats", get(get_stats))
        .route("/latest", get(get_latest))
}

fn api_error(status: StatusCode, message: impl Into<String>) -> (StatusCode, Json<Value>) {
    (status, Json(json!({ "error": message.into() })))
}

async fn get_transactions(
    auth: AuthUser,
    State(pool): State<DbPool>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Transaction>>, (StatusCode, Json<Value>)> {
    let limit = query.limit.unwrap_or(50).clamp(0, 200);
    let offset = query.offset.unwrap_or(0).max(0);
    let mut conn = pool
        .get()
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let rows = transactions::table
        .filter(transactions::merchant_id.eq(auth.0))
        .order(transactions::detected_at.desc())
        .limit(limit)
        .offset(offset)
        .load::<Transaction>(&mut conn)
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(Json(rows))
}

async fn get_stats(
    auth: AuthUser,
    State(pool): State<DbPool>,
) -> Result<Json<Stats>, (StatusCode, Json<Value>)> {
    let mut conn = pool
        .get()
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    merchants::table
        .find(auth.0.clone())
        .get_result::<Merchant>(&mut conn)
        .await
        .map_err(|_| api_error(StatusCode::NOT_FOUND, "Not onboarded"))?;

    let confirmed = transactions::table
        .filter(transactions::merchant_id.eq(auth.0.clone()))
        .filter(transactions::status.eq("confirmed"));

    let all_rows = confirmed
        .select(transactions::amount)
        .load::<bigdecimal::BigDecimal>(&mut conn)
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let total_revenue = all_rows.iter().filter_map(ToPrimitive::to_f64).sum::<f64>();
    let order_count = all_rows.len() as i64;

    let today_start = Utc::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    let week_start = Utc::now() - Duration::days(7);

    let today_rows = transactions::table
        .filter(transactions::merchant_id.eq(auth.0.clone()))
        .filter(transactions::status.eq("confirmed"))
        .filter(transactions::detected_at.ge(today_start))
        .select(transactions::amount)
        .load::<bigdecimal::BigDecimal>(&mut conn)
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let week_rows = transactions::table
        .filter(transactions::merchant_id.eq(auth.0))
        .filter(transactions::status.eq("confirmed"))
        .filter(transactions::detected_at.ge(week_start))
        .select(transactions::amount)
        .load::<bigdecimal::BigDecimal>(&mut conn)
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(Json(Stats {
        total_revenue,
        order_count,
        avg_order_value: if order_count > 0 {
            total_revenue / order_count as f64
        } else {
            0.0
        },
        today_revenue: today_rows.iter().filter_map(ToPrimitive::to_f64).sum(),
        today_orders: today_rows.len() as i64,
        week_revenue: week_rows.iter().filter_map(ToPrimitive::to_f64).sum(),
        week_orders: week_rows.len() as i64,
    }))
}

async fn get_latest(
    auth: AuthUser,
    State(pool): State<DbPool>,
) -> Result<Json<LatestResponse>, (StatusCode, Json<Value>)> {
    let mut conn = pool
        .get()
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let row = transactions::table
        .filter(transactions::merchant_id.eq(auth.0))
        .order(transactions::detected_at.desc())
        .select((transactions::id, transactions::detected_at))
        .first::<(String, chrono::DateTime<Utc>)>(&mut conn)
        .await
        .optional()
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(Json(match row {
        Some((latest_id, latest_at)) => LatestResponse {
            latest_id: Some(latest_id),
            latest_at: Some(latest_at),
        },
        None => LatestResponse {
            latest_id: None,
            latest_at: None,
        },
    }))
}
