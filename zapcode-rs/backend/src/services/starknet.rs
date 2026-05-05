use anyhow::Result;
use starknet::core::types::Felt;
use starkzap_rs::{
    signer::StarkSigner, tokens, Amount, Network, OnboardConfig, Recipient, StarkZap,
    StarkZapConfig,
};

pub const BUYER_PREFUND: &str = "400000000000000000";
pub const MERCHANT_PREFUND: &str = "400000000000000000";

pub fn network() -> String {
    std::env::var("STARKNET_NETWORK").unwrap_or_else(|_| "mainnet".to_string())
}

pub fn usdc_address() -> String {
    if network() != "mainnet" {
        if let Ok(address) = std::env::var("ZUSDC_ADDRESS") {
            let address = address.trim();
            if !address.is_empty() {
                return address.to_string();
            }
        }
    }

    if network() == "mainnet" {
        "0x033068f6539f8e6e6b131e6b2b814e6c34a5224bc66947c47dab9dfee93b35fb".to_string()
    } else {
        "0x053b40a647cedfca6ca84f542a0fe36736031905a9639a7f19a3c1e66bfd5080".to_string()
    }
}

pub fn usdce_address() -> Option<&'static str> {
    if network() == "mainnet" {
        Some("0x053c91253bc9682c04929ca02ed00b3e423f6710d2ee7e0d5ebb06f3ecf368a8")
    } else {
        None
    }
}

pub async fn send_strk(to_address: &str, amount: &str, label: &str) -> Result<()> {
    let treasury_address = std::env::var("TREASURY_ADDRESS").unwrap_or_default();
    let treasury_priv_key = std::env::var("TREASURY_PRIVATE_KEY").unwrap_or_default();

    if treasury_address.is_empty() || treasury_priv_key.is_empty() {
        tracing::warn!(
            "TREASURY_ADDRESS or TREASURY_PRIVATE_KEY not configured, skipping STRK prefund"
        );
        return Ok(());
    }

    let network = if network() == "mainnet" {
        Network::Mainnet
    } else {
        Network::Sepolia
    };
    let rpc_url = std::env::var("STARKNET_RPC_URL")
        .or_else(|_| std::env::var("RPC_URL"))
        .ok();
    let config = match network {
        Network::Mainnet => StarkZapConfig::mainnet(),
        Network::Sepolia => StarkZapConfig::sepolia(),
        Network::Devnet => StarkZapConfig::devnet(),
    };
    let config = if let Some(rpc_url) = rpc_url {
        config.with_rpc(rpc_url)
    } else {
        config
    };
    let sdk = StarkZap::new(config);
    let signer = StarkSigner::new(&treasury_priv_key, &treasury_address)?;
    let wallet = sdk.onboard(OnboardConfig::Signer(signer)).await?;
    let strk = tokens::by_symbol(network, "STRK")
        .ok_or_else(|| anyhow::anyhow!("STRK token preset not found"))?;
    let raw = amount.parse::<u128>()?;
    let recipient = Felt::from_hex(to_address).map_err(|err| anyhow::anyhow!(err.to_string()))?;
    let tx = wallet
        .transfer(
            &strk,
            vec![Recipient::new(recipient, Amount::from_raw(raw, &strk))],
        )
        .await?;
    tracing::info!(
        "[treasury] {} sent to {} - tx: {}",
        label,
        to_address,
        tx.hash_hex()
    );
    tx.wait().await?;
    tracing::info!("[treasury] {} confirmed - {} funded", label, to_address);
    Ok(())
}
