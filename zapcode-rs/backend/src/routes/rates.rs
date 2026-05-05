use axum::{extract::Query, http::StatusCode, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::db::DbPool;
use crate::services::rates::fetch_rate;

#[derive(Deserialize)]
struct RateQuery {
    from: Option<String>,
    to: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RateResponse {
    from: String,
    to: String,
    rate: f64,
    fetched_at: i64,
    age_seconds: i64,
}

pub fn router() -> Router<DbPool> {
    Router::new().route("/", get(get_rate))
}

async fn get_rate(
    Query(query): Query<RateQuery>,
) -> Result<Json<RateResponse>, (StatusCode, Json<Value>)> {
    let from = query
        .from
        .unwrap_or_else(|| "USDC".to_string())
        .to_uppercase();
    let to = query.to.unwrap_or_else(|| "USD".to_string()).to_uppercase();
    let result = fetch_rate(&from, &to).await.map_err(|err| {
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": format!("Rate fetch failed: {}", err) })),
        )
    })?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(result.fetched_at);

    Ok(Json(RateResponse {
        from,
        to,
        rate: result.rate,
        fetched_at: result.fetched_at,
        age_seconds: now - result.fetched_at,
    }))
}
