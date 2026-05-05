use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use starknet::core::{
    types::{BlockId, BlockTag, Call, Felt, FunctionCall},
    utils::get_selector_from_name,
};
use starknet::providers::Provider;
use starkzap_rs::{
    paymaster::{FeeMode, PaymasterConfig},
    signer::PrivySigner,
    AccountPreset, DeployMode, EnsureReadyOptions, ExecuteOptions, Network, OnboardConfig,
    StarkZap, StarkZapConfig,
};

use crate::{
    db::{
        schema::{buyers, merchants},
        DbPool,
    },
    models::{Buyer, Merchant},
    services::{
        privy::{create_starknet_wallet, sign_hash},
        starknet::{send_strk, BUYER_PREFUND},
    },
    utils::auth::AuthUser,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WalletResponse {
    wallet: WalletPayload,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WalletPayload {
    id: String,
    address: String,
    public_key: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SignRequest {
    wallet_id: String,
    hash: String,
}

#[derive(Deserialize)]
struct UploadLogoRequest {
    image: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FaucetResponse {
    address: String,
    amount: String,
    token_address: String,
    transaction_hash: String,
}

pub fn router() -> Router<DbPool> {
    Router::new()
        .route("/starknet", post(post_starknet))
        .route("/sign", post(post_sign))
        .route("/faucet", post(post_faucet))
        .route("/paymaster/execute", post(post_paymaster_execute))
        .route("/paymaster", post(post_paymaster))
        .route("/paymaster/deploy", post(post_paymaster_deploy))
        .route("/upload-logo", post(post_upload_logo))
}

fn api_error(status: StatusCode, message: impl Into<String>) -> (StatusCode, Json<Value>) {
    (status, Json(json!({ "error": message.into() })))
}

fn wallet_response(id: String, address: String, public_key: String) -> Json<WalletResponse> {
    Json(WalletResponse {
        wallet: WalletPayload {
            id,
            address,
            public_key,
        },
    })
}

fn is_mainnet() -> bool {
    std::env::var("STARKNET_NETWORK").unwrap_or_else(|_| "sepolia".to_string()) == "mainnet"
}

fn starkzap_network() -> Network {
    if is_mainnet() {
        Network::Mainnet
    } else {
        Network::Sepolia
    }
}

async fn post_starknet(
    auth: AuthUser,
    State(pool): State<DbPool>,
) -> Result<Json<WalletResponse>, (StatusCode, Json<Value>)> {
    let mut conn = pool
        .get()
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    if let Some(merchant) = merchants::table
        .find(auth.0.clone())
        .get_result::<Merchant>(&mut conn)
        .await
        .optional()
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
    {
        return Ok(wallet_response(
            merchant.wallet_id,
            merchant.wallet_address,
            merchant.public_key,
        ));
    }

    if let Some(buyer) = buyers::table
        .find(auth.0.clone())
        .get_result::<Buyer>(&mut conn)
        .await
        .optional()
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
    {
        return Ok(wallet_response(
            buyer.wallet_id,
            buyer.wallet_address,
            buyer.public_key,
        ));
    }

    let wallet = create_starknet_wallet(&auth.0)
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let public_key = wallet.public_key.unwrap_or_default();

    send_strk(&wallet.address, BUYER_PREFUND, "buyer")
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let buyer = Buyer {
        id: auth.0.clone(),
        wallet_id: wallet.id.clone(),
        wallet_address: wallet.address.clone(),
        public_key: public_key.clone(),
        network: if is_mainnet() { "mainnet" } else { "sepolia" }.to_string(),
        created_at: Utc::now(),
    };

    diesel::insert_into(buyers::table)
        .values(&buyer)
        .on_conflict_do_nothing()
        .execute(&mut conn)
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let inserted = buyers::table
        .find(auth.0)
        .get_result::<Buyer>(&mut conn)
        .await
        .optional()
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(match inserted {
        Some(row) => wallet_response(row.wallet_id, row.wallet_address, row.public_key),
        None => wallet_response(wallet.id, wallet.address, public_key),
    })
}

async fn post_sign(
    Json(body): Json<SignRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if body.wallet_id.trim().is_empty() || body.hash.trim().is_empty() {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "walletId and hash are required",
        ));
    }

    let signature = sign_hash(&body.wallet_id, &body.hash)
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(Json(json!({ "signature": signature })))
}

async fn post_faucet(
    auth: AuthUser,
    State(pool): State<DbPool>,
) -> Result<Json<FaucetResponse>, (StatusCode, Json<Value>)> {
    let token_address = std::env::var("ZUSDC_ADDRESS").map_err(|_| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ZUSDC_ADDRESS is not configured",
        )
    })?;

    let mut conn = pool
        .get()
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let merchant = merchants::table
        .find(auth.0)
        .get_result::<Merchant>(&mut conn)
        .await
        .map_err(|_| api_error(StatusCode::NOT_FOUND, "Merchant not onboarded yet"))?;
    drop(conn);

    let address = parse_felt(&merchant.wallet_address, "address")?;
    let public_key = parse_felt(&merchant.public_key, "publicKey")?;
    let token = parse_felt(&token_address, "ZUSDC_ADDRESS")?;

    let signer = PrivySigner::from_env()
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let signer = signer.with_wallet_and_public_key(merchant.wallet_id.clone(), address, public_key);

    let network = starkzap_network();
    let config = match network {
        Network::Mainnet => StarkZapConfig::mainnet(),
        Network::Sepolia => StarkZapConfig::sepolia(),
        Network::Devnet => StarkZapConfig::devnet(),
    };
    let config = if let Ok(rpc_url) =
        std::env::var("STARKNET_RPC_URL").or_else(|_| std::env::var("RPC_URL"))
    {
        config.with_rpc(rpc_url)
    } else {
        config
    };

    let sdk = StarkZap::new(config);
    let wallet = sdk
        .onboard(OnboardConfig::PrivyWithPreset(
            signer,
            AccountPreset::ArgentXV050,
        ))
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let deployed = wallet
        .is_deployed()
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    if !deployed {
        tracing::info!(
            wallet = %merchant.wallet_address,
            amount = BUYER_PREFUND,
            "prefunding undeployed merchant account before faucet mint"
        );
        send_strk(
            &merchant.wallet_address,
            BUYER_PREFUND,
            "faucet account deploy",
        )
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    }

    wallet
        .ensure_ready_with_options(
            EnsureReadyOptions {
                deploy: DeployMode::IfNeeded,
                fee_mode: Some(FeeMode::UserPays),
            },
            None::<fn(starkzap_rs::ProgressEvent)>,
        )
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let selector = get_selector_from_name("mint")
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let tx = wallet
        .execute_with_options(
            vec![Call {
                to: token,
                selector,
                calldata: vec![address, Felt::from(10_000_000_u64), Felt::ZERO],
            }],
            ExecuteOptions {
                fee_mode: Some(FeeMode::Paymaster(PaymasterConfig::from_env())),
            },
        )
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    tx.wait()
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(Json(FaucetResponse {
        address: merchant.wallet_address,
        amount: "10".to_string(),
        token_address,
        transaction_hash: tx.hash_hex(),
    }))
}

async fn post_paymaster(
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    reject_paymaster_proxy(body).await
}

async fn post_paymaster_deploy(
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    reject_paymaster_proxy(body).await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SponsoredExecuteRequest {
    wallet_id: String,
    address: String,
    public_key: String,
    calls: Vec<SponsoredCall>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SponsoredCall {
    #[serde(alias = "contractAddress", alias = "to")]
    contract_address: String,
    #[serde(default)]
    entrypoint: Option<String>,
    #[serde(default)]
    selector: Option<String>,
    #[serde(default)]
    calldata: Vec<String>,
}

async fn reject_paymaster_proxy(
    _body: Value,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    Ok((
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "Direct AVNU paymaster proxy disabled. Use /api/wallet/paymaster/execute so sponsored transactions go through starkzap-rs."
        })),
    ))
}

async fn post_paymaster_execute(
    Json(body): Json<SponsoredExecuteRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if body.wallet_id.trim().is_empty()
        || body.address.trim().is_empty()
        || body.public_key.trim().is_empty()
        || body.calls.is_empty()
    {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "walletId, address, publicKey, and calls are required",
        ));
    }

    let address = parse_felt(&body.address, "address")?;
    let public_key = parse_felt(&body.public_key, "publicKey")?;
    let signer = PrivySigner::from_env()
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
        .with_wallet_and_public_key(body.wallet_id, address, public_key);

    let network = starkzap_network();
    let config = match network {
        Network::Mainnet => StarkZapConfig::mainnet(),
        Network::Sepolia => StarkZapConfig::sepolia(),
        Network::Devnet => StarkZapConfig::devnet(),
    };
    let config = if let Ok(rpc_url) =
        std::env::var("STARKNET_RPC_URL").or_else(|_| std::env::var("RPC_URL"))
    {
        config.with_rpc(rpc_url)
    } else {
        config
    };

    let sdk = StarkZap::new(config);
    let wallet = sdk
        .onboard(OnboardConfig::PrivyWithPreset(
            signer,
            AccountPreset::ArgentXV050,
        ))
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let deployed = wallet
        .is_deployed()
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    if !deployed {
        tracing::info!(
            wallet = %body.address,
            amount = BUYER_PREFUND,
            "prefunding undeployed account before user-pays deployment"
        );
        send_strk(&body.address, BUYER_PREFUND, "account deploy")
            .await
            .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    }

    wallet
        .ensure_ready_with_options(
            EnsureReadyOptions {
                deploy: DeployMode::IfNeeded,
                fee_mode: Some(FeeMode::UserPays),
            },
            None::<fn(starkzap_rs::ProgressEvent)>,
        )
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let usdc_address = configured_usdc_address()?;
    let calls = body
        .calls
        .iter()
        .map(|call| parse_sponsored_call(call, &usdc_address))
        .collect::<Result<Vec<_>, _>>()?;
    validate_token_balances(&sdk, address, &calls).await?;

    tracing::info!(
        wallet = %body.address,
        calls = calls.len(),
        token_override = %usdc_address,
        "submitting sponsored execute"
    );

    let tx = wallet
        .execute_with_options(
            calls,
            ExecuteOptions {
                fee_mode: Some(FeeMode::Paymaster(PaymasterConfig::from_env())),
            },
        )
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(Json(json!({
        "transactionHash": tx.hash_hex(),
        "hash": tx.hash_hex(),
    })))
}

fn configured_usdc_address() -> Result<String, (StatusCode, Json<Value>)> {
    std::env::var("ZUSDC_ADDRESS")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "ZUSDC_ADDRESS is not configured",
            )
        })
}

fn parse_sponsored_call(
    call: &SponsoredCall,
    usdc_address: &str,
) -> Result<Call, (StatusCode, Json<Value>)> {
    let selector = match (&call.selector, &call.entrypoint) {
        (Some(selector), _) => parse_felt(selector, "selector")?,
        (None, Some(entrypoint)) => get_selector_from_name(entrypoint)
            .map_err(|err| api_error(StatusCode::BAD_REQUEST, err.to_string()))?,
        (None, None) => {
            return Err(api_error(
                StatusCode::BAD_REQUEST,
                "selector or entrypoint is required",
            ));
        }
    };
    let transfer_selector = get_selector_from_name("transfer")
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let is_token_transfer = selector == transfer_selector && call.calldata.len() == 3;
    let contract_address = if is_token_transfer {
        usdc_address
    } else {
        &call.contract_address
    };
    let to = parse_felt(contract_address, "contractAddress")?;
    let calldata = call
        .calldata
        .iter()
        .map(|value| parse_felt(value, "calldata"))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Call {
        to,
        selector,
        calldata,
    })
}

fn parse_felt(value: &str, field: &str) -> Result<Felt, (StatusCode, Json<Value>)> {
    let trimmed = value.trim();
    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        return Felt::from_hex(trimmed).map_err(|err| {
            api_error(
                StatusCode::BAD_REQUEST,
                format!("Invalid {}: {}", field, err),
            )
        });
    }

    let parsed = trimmed.parse::<u128>().map_err(|err| {
        api_error(
            StatusCode::BAD_REQUEST,
            format!("Invalid {}: {}", field, err),
        )
    })?;
    Ok(Felt::from(parsed))
}

async fn validate_token_balances(
    sdk: &StarkZap,
    owner: Felt,
    calls: &[Call],
) -> Result<(), (StatusCode, Json<Value>)> {
    let transfer_selector = get_selector_from_name("transfer")
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    let balance_selector = get_selector_from_name("balance_of")
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    for call in calls
        .iter()
        .filter(|call| call.selector == transfer_selector && call.calldata.len() == 3)
    {
        let balance = sdk
            .provider()
            .call(
                FunctionCall {
                    contract_address: call.to,
                    entry_point_selector: balance_selector,
                    calldata: vec![owner],
                },
                BlockId::Tag(BlockTag::Latest),
            )
            .await
            .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

        let balance_low = balance.as_slice().first().copied().unwrap_or(Felt::ZERO);
        let balance_high = balance.get(1).copied().unwrap_or(Felt::ZERO);
        let amount_low = call.calldata[1];
        let amount_high = call.calldata[2];

        tracing::info!(
            owner = %format!("{:#x}", owner),
            token = %format!("{:#x}", call.to),
            balance_low = %format!("{:#x}", balance_low),
            balance_high = %format!("{:#x}", balance_high),
            amount_low = %format!("{:#x}", amount_low),
            amount_high = %format!("{:#x}", amount_high),
            "validated ERC20 transfer balance"
        );

        if compare_u256(balance_low, balance_high, amount_low, amount_high)? < 0 {
            return Err(api_error(
                StatusCode::BAD_REQUEST,
                format!(
                    "Insufficient USDC balance on token {:#x}: have {} raw units, need {} raw units",
                    call.to,
                    felt_to_u128(balance_low, "balance")?,
                    felt_to_u128(amount_low, "amount")?
                ),
            ));
        }
    }

    Ok(())
}

fn compare_u256(
    left_low: Felt,
    left_high: Felt,
    right_low: Felt,
    right_high: Felt,
) -> Result<i8, (StatusCode, Json<Value>)> {
    let left_high = felt_to_u128(left_high, "balance high")?;
    let right_high = felt_to_u128(right_high, "amount high")?;
    if left_high != right_high {
        return Ok(if left_high < right_high { -1 } else { 1 });
    }

    let left_low = felt_to_u128(left_low, "balance low")?;
    let right_low = felt_to_u128(right_low, "amount low")?;
    Ok(if left_low < right_low {
        -1
    } else if left_low > right_low {
        1
    } else {
        0
    })
}

fn felt_to_u128(value: Felt, field: &str) -> Result<u128, (StatusCode, Json<Value>)> {
    let bytes = value.to_bytes_le();
    if bytes.iter().skip(16).any(|byte| *byte != 0) {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            format!("{} does not fit in u128", field),
        ));
    }

    let mut raw = [0_u8; 16];
    raw.copy_from_slice(&bytes[..16]);
    Ok(u128::from_le_bytes(raw))
}

async fn post_upload_logo(
    _auth: AuthUser,
    Json(body): Json<UploadLogoRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if body.image.trim().is_empty() {
        return Err(api_error(StatusCode::BAD_REQUEST, "image is required"));
    }
    let key = std::env::var("IMGBB_API_KEY").map_err(|_| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "IMGBB_API_KEY not configured",
        )
    })?;

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.imgbb.com/1/upload")
        .form(&[("key", key), ("image", body.image)])
        .send()
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    let data = response
        .json::<Value>()
        .await
        .map_err(|err| api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    if !data
        .get("success")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        let message = data
            .pointer("/error/message")
            .and_then(Value::as_str)
            .unwrap_or("imgbb upload failed");
        return Err(api_error(StatusCode::INTERNAL_SERVER_ERROR, message));
    }

    let url = data
        .pointer("/data/display_url")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "imgbb upload missing display_url",
            )
        })?;
    Ok(Json(json!({ "url": url })))
}
