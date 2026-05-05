use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    db::{schema::merchants, DbPool},
    models::Merchant,
    services::{
        email::send_welcome,
        privy::{create_starknet_wallet, get_user_email, privy_client},
        qr::{build_qr_payload, generate_qr_png, GenerateQrOptions},
        rates::{convert, ConvertedAmount},
        starknet::{send_strk, MERCHANT_PREFUND},
    },
    utils::auth::AuthUser,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OnboardRequest {
    business_name: String,
    currency: Option<String>,
    country: Option<String>,
    network: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateMerchantRequest {
    business_name: Option<String>,
    currency: Option<String>,
    country: Option<String>,
    logo_url: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MerchantResponse {
    #[serde(flatten)]
    merchant: Merchant,
    fx: Option<MerchantFx>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PublicMerchantResponse {
    id: String,
    business_name: String,
    wallet_address: String,
    currency: String,
    network: String,
    logo_url: Option<String>,
    fx: Option<MerchantFx>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MerchantFx {
    rate: f64,
    currency: String,
    fetched_at: i64,
    age_seconds: i64,
}

pub fn router() -> Router<DbPool> {
    Router::new()
        .route("/me", get(get_me).patch(patch_me))
        .route("/onboard", post(post_onboard))
        .route("/:id", get(get_merchant))
        .route("/:id/qr.png", get(get_merchant_qr))
}

fn api_error(
    status: StatusCode,
    message: impl Into<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(json!({ "error": message.into() })))
}

fn merchant_fx(converted: ConvertedAmount) -> MerchantFx {
    MerchantFx {
        rate: converted.rate,
        currency: converted.currency,
        fetched_at: converted.fetched_at,
        age_seconds: converted.age_seconds,
    }
}

async fn fx_for(currency: &str) -> Option<MerchantFx> {
    convert(1.0, currency).await.ok().flatten().map(merchant_fx)
}

fn padded_address(raw: &str) -> String {
    if raw.starts_with("0x") {
        format!("0x{:0>64}", raw.trim_start_matches("0x").to_lowercase())
    } else {
        raw.to_string()
    }
}

async fn get_me(
    auth: AuthUser,
    State(pool): State<DbPool>,
) -> Result<Json<MerchantResponse>, (StatusCode, Json<serde_json::Value>)> {
    let mut conn = pool
        .get()
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let merchant = merchants::table
        .find(auth.0)
        .get_result::<Merchant>(&mut conn)
        .await
        .map_err(|_| api_error(StatusCode::NOT_FOUND, "Not onboarded yet"))?;

    let fx = fx_for(&merchant.currency).await;
    Ok(Json(MerchantResponse { merchant, fx }))
}

async fn post_onboard(
    auth: AuthUser,
    State(pool): State<DbPool>,
    Json(body): Json<OnboardRequest>,
) -> Result<(StatusCode, Json<MerchantResponse>), (StatusCode, Json<serde_json::Value>)> {
    if body.business_name.trim().is_empty() {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "businessName is required",
        ));
    }

    let mut conn = pool
        .get()
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let existing = merchants::table
        .find(auth.0.clone())
        .select(merchants::id)
        .first::<String>(&mut conn)
        .await
        .optional()
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    if existing.is_some() {
        return Err(api_error(StatusCode::CONFLICT, "Already onboarded"));
    }

    let privy = privy_client()
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let email = get_user_email(&privy, &auth.0)
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let wallet = create_starknet_wallet(&auth.0)
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let public_key = wallet
        .public_key
        .filter(|key| !key.is_empty())
        .ok_or_else(|| {
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not retrieve wallet publicKey from Privy",
            )
        })?;

    let new_merchant = Merchant {
        id: auth.0,
        email,
        business_name: body.business_name,
        wallet_id: wallet.id,
        wallet_address: wallet.address,
        public_key,
        currency: body
            .currency
            .unwrap_or_else(|| "USD".to_string())
            .to_uppercase(),
        country: body
            .country
            .unwrap_or_else(|| "KE".to_string())
            .to_uppercase(),
        network: body.network.unwrap_or_else(|| "sepolia".to_string()),
        logo_url: None,
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let merchant = diesel::insert_into(merchants::table)
        .values(&new_merchant)
        .get_result::<Merchant>(&mut conn)
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let prefund_address = merchant.wallet_address.clone();
    tokio::spawn(async move {
        let _ = send_strk(&prefund_address, MERCHANT_PREFUND, "merchant").await;
    });

    let welcome_merchant = merchant.clone();
    tokio::spawn(async move {
        let _ = send_welcome(&welcome_merchant).await;
    });

    let fx = fx_for(&merchant.currency).await;
    Ok((StatusCode::CREATED, Json(MerchantResponse { merchant, fx })))
}

async fn patch_me(
    auth: AuthUser,
    State(pool): State<DbPool>,
    Json(body): Json<UpdateMerchantRequest>,
) -> Result<Json<MerchantResponse>, (StatusCode, Json<serde_json::Value>)> {
    if body.business_name.is_none()
        && body.currency.is_none()
        && body.country.is_none()
        && body.logo_url.is_none()
    {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "No valid fields to update",
        ));
    }

    let mut conn = pool
        .get()
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let current = merchants::table
        .find(auth.0.clone())
        .get_result::<Merchant>(&mut conn)
        .await
        .map_err(|_| api_error(StatusCode::NOT_FOUND, "Merchant not found"))?;

    let merchant = diesel::update(merchants::table.find(auth.0))
        .set((
            merchants::business_name.eq(body.business_name.unwrap_or(current.business_name)),
            merchants::currency.eq(body.currency.unwrap_or(current.currency).to_uppercase()),
            merchants::country.eq(body.country.unwrap_or(current.country).to_uppercase()),
            merchants::logo_url.eq(body.logo_url.or(current.logo_url)),
            merchants::updated_at.eq(Utc::now()),
        ))
        .get_result::<Merchant>(&mut conn)
        .await
        .map_err(|_| api_error(StatusCode::NOT_FOUND, "Merchant not found"))?;

    let fx = fx_for(&merchant.currency).await;
    Ok(Json(MerchantResponse { merchant, fx }))
}

async fn get_merchant(
    Path(raw_id): Path<String>,
    State(pool): State<DbPool>,
) -> Result<Json<PublicMerchantResponse>, (StatusCode, Json<serde_json::Value>)> {
    let padded = padded_address(&raw_id);
    let mut conn = pool
        .get()
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let merchant = merchants::table
        .filter(
            merchants::wallet_address
                .eq(&raw_id)
                .or(merchants::wallet_address.eq(&padded)),
        )
        .get_result::<Merchant>(&mut conn)
        .await
        .map_err(|_| api_error(StatusCode::NOT_FOUND, "Merchant not found"))?;

    let fx = fx_for(&merchant.currency).await;
    Ok(Json(PublicMerchantResponse {
        id: merchant.id,
        business_name: merchant.business_name,
        wallet_address: merchant.wallet_address,
        currency: merchant.currency,
        network: merchant.network,
        logo_url: merchant.logo_url,
        fx,
    }))
}

async fn get_merchant_qr(
    Path(raw_id): Path<String>,
    State(pool): State<DbPool>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let padded = padded_address(&raw_id);
    let mut conn = pool
        .get()
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let merchant = merchants::table
        .filter(
            merchants::wallet_address
                .eq(&raw_id)
                .or(merchants::wallet_address.eq(&padded)),
        )
        .get_result::<Merchant>(&mut conn)
        .await
        .map_err(|_| api_error(StatusCode::NOT_FOUND, "Merchant not found"))?;

    let payload = build_qr_payload(&merchant.wallet_address, &merchant.network, &merchant.id);
    let png = generate_qr_png(
        &payload,
        Some(GenerateQrOptions {
            business_name: Some(merchant.business_name.clone()),
            logo_url: merchant.logo_url.clone(),
        }),
    )
    .await
    .map_err(|_| api_error(StatusCode::INTERNAL_SERVER_ERROR, "QR generation failed"))?;

    let mut response = (StatusCode::OK, png).into_response();
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, "image/png".parse().unwrap());
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, "no-cache".parse().unwrap());
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        format!(
            "attachment; filename=\"{}-zapcode.png\"",
            merchant.business_name.replace(char::is_whitespace, "-")
        )
        .parse()
        .unwrap(),
    );
    Ok(response)
}
