use anyhow::{anyhow, Result};
use privy::{config::PrivyConfig, Privy};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use starknet::core::types::Felt;
use starkzap_rs::signer::PrivySigner;
use std::sync::OnceLock;

static PRIVY_VERIFICATION_KEY: OnceLock<String> = OnceLock::new();

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WalletDetails {
    pub id: String,
    pub address: String,
    pub public_key: Option<String>,
}

#[derive(Deserialize)]
struct AppSettingsResponse {
    verification_key: String,
}

async fn fetch_verification_key(app_id: &str, app_secret: &str) -> Result<String> {
    let mut headers = HeaderMap::new();
    headers.insert("privy-app-id", app_id.parse()?);

    let api_url = std::env::var("PRIVY_API_URL").unwrap_or_else(|_| "https://auth.privy.io".into());
    let url = format!("{}/api/v1/apps/{}", api_url.trim_end_matches('/'), app_id);
    let settings = reqwest::Client::new()
        .get(url)
        .basic_auth(app_id, Some(app_secret))
        .headers(headers)
        .send()
        .await?
        .error_for_status()?
        .json::<AppSettingsResponse>()
        .await?;

    Ok(settings.verification_key)
}

pub async fn privy_config() -> Result<PrivyConfig> {
    let app_id = std::env::var("PRIVY_APP_ID")
        .map_err(|_| anyhow!("Missing required environment variable: PRIVY_APP_ID"))?;
    let app_secret = std::env::var("PRIVY_APP_SECRET")
        .map_err(|_| anyhow!("Missing required environment variable: PRIVY_APP_SECRET"))?;
    let verification_key = match std::env::var("PRIVY_VERIFICATION_KEY") {
        Ok(key) => key,
        Err(_) => {
            if let Some(key) = PRIVY_VERIFICATION_KEY.get() {
                key.clone()
            } else {
                let key = fetch_verification_key(&app_id, &app_secret).await?;
                let _ = PRIVY_VERIFICATION_KEY.set(key.clone());
                key
            }
        }
    };

    Ok(PrivyConfig {
        app_id,
        app_secret,
        verification_key,
    })
}

pub async fn privy_client() -> Result<Privy> {
    let config = privy_config().await?;
    Ok(Privy::new(config))
}

pub async fn verify_auth_token(privy: &Privy, token: &str) -> Result<String> {
    let session = privy
        .authenticate_user(token)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    Ok(session.user_id)
}

pub async fn get_user_email(privy: &Privy, user_id: &str) -> Result<String> {
    privy
        .get_email_by_user_id(user_id)
        .await
        .map_err(|e| anyhow!(e.to_string()))
}

pub fn init_privy_signer() -> Result<PrivySigner> {
    PrivySigner::from_env().map_err(|e| anyhow!(e.to_string()))
}

pub async fn create_starknet_wallet(user_id: &str) -> Result<WalletDetails> {
    let mut signer = init_privy_signer()?;
    let wallet_info = signer
        .create_wallet_info(user_id)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;
    Ok(WalletDetails {
        id: wallet_info.wallet_id,
        address: format!("{:#x}", wallet_info.address),
        public_key: wallet_info.public_key.map(|p| format!("{:#x}", p)),
    })
}

pub async fn sign_hash(wallet_id: &str, hash: &str) -> Result<String> {
    let mut signer = init_privy_signer()?;
    if signer.wallet_id().is_none() {
        signer = signer.with_wallet(wallet_id.to_string(), Felt::ZERO);
    }

    let felt = Felt::from_hex(hash).map_err(|e| anyhow!(e.to_string()))?;
    let (r, s) = signer
        .sign_hash(felt)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;

    Ok(format!("{:x}{:x}", r, s))
}
