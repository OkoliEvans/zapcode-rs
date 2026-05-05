use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde_json::{json, Value};

use crate::{
    db::{
        schema::{merchants, transactions},
        DbPool,
    },
    models::PublicStats,
};

pub fn router() -> Router<DbPool> {
    Router::new().route("/public", get(get_public_stats))
}

fn api_error(status: StatusCode, message: impl Into<String>) -> (StatusCode, Json<Value>) {
    (status, Json(json!({ "error": message.into() })))
}

async fn get_public_stats(
    State(pool): State<DbPool>,
) -> Result<Json<PublicStats>, (StatusCode, Json<Value>)> {
    let mut conn = pool
        .get()
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let merchant_count = merchants::table
        .count()
        .get_result::<i64>(&mut conn)
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let payment_count = transactions::table
        .filter(transactions::status.eq("confirmed"))
        .count()
        .get_result::<i64>(&mut conn)
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let senders = transactions::table
        .select(transactions::from_address)
        .distinct()
        .load::<String>(&mut conn)
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(Json(PublicStats {
        merchants: merchant_count,
        payments: payment_count,
        unique_senders: senders.len() as i64,
    }))
}
