use std::collections::HashMap;

use anyhow::{anyhow, Result};
use bigdecimal::BigDecimal;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use starknet::{
    core::{
        types::{BlockId, EmittedEvent, EventFilter, Felt},
        utils::get_selector_from_name,
    },
    providers::{
        jsonrpc::{HttpTransport, JsonRpcClient},
        Provider,
    },
};
use uuid::Uuid;

use crate::{
    db::{
        schema::{merchants, transactions},
        DbPool,
    },
    models::{Merchant, Transaction},
    services::{
        email::send_payment_received,
        rates::convert,
        starknet::{usdc_address, usdce_address},
    },
};

const START_BLOCK: u64 = 7_702_278;

#[derive(Debug, Clone)]
struct ParsedTransfer {
    from: String,
    to: String,
    amount: BigDecimal,
}

#[derive(Debug, Default)]
pub struct Watcher {
    last_checked_block: Option<u64>,
}

impl Watcher {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn poll_once(&mut self, pool: &DbPool) -> Result<()> {
        let provider = provider()?;
        let latest_block = provider.block_number().await?;

        let Some(last_checked_block) = self.last_checked_block else {
            self.last_checked_block = Some(START_BLOCK);
            tracing::info!("[watcher] starting from block {}", START_BLOCK);
            return Ok(());
        };

        if latest_block <= last_checked_block {
            return Ok(());
        }

        let from_block = last_checked_block + 1;
        let to_block = latest_block;
        let active_merchants = active_merchants(pool).await?;
        if active_merchants.is_empty() {
            self.last_checked_block = Some(latest_block);
            return Ok(());
        }

        let address_set = active_merchants
            .into_iter()
            .map(|merchant| (normalize_addr(&merchant.wallet_address), merchant.id))
            .collect::<HashMap<_, _>>();

        let usdc = usdc_address();
        process_contract(
            pool,
            &provider,
            &usdc,
            "USDC",
            from_block,
            to_block,
            &address_set,
        )
        .await?;
        if let Some(address) = usdce_address() {
            process_contract(
                pool,
                &provider,
                address,
                "USDC.e",
                from_block,
                to_block,
                &address_set,
            )
            .await?;
        }

        self.last_checked_block = Some(latest_block);
        Ok(())
    }
}

fn provider() -> Result<JsonRpcClient<HttpTransport>> {
    let rpc_url =
        std::env::var("STARKNET_RPC_URL").map_err(|_| anyhow!("STARKNET_RPC_URL is required"))?;
    let url = reqwest::Url::parse(&rpc_url)?;
    Ok(JsonRpcClient::new(HttpTransport::new(url)))
}

fn normalize_addr(addr: &str) -> String {
    format!("0x{:0>64}", addr.trim_start_matches("0x").to_lowercase())
}

fn is_zero_addr(addr: &str) -> bool {
    normalize_addr(addr) == format!("0x{:0>64}", "")
}

fn felt_hex(felt: Felt) -> String {
    format!("{:#x}", felt)
}

fn felt_to_biguint(felt: Felt) -> BigUint {
    BigUint::from_bytes_be(&felt.to_bytes_be())
}

fn parse_transfer_event(event: &EmittedEvent) -> Option<ParsedTransfer> {
    let (from, to, amount_low, amount_high) = if event.keys.len() >= 3 && event.data.len() >= 2 {
        (event.keys[1], event.keys[2], event.data[0], event.data[1])
    } else if event.data.len() >= 4 {
        (event.data[0], event.data[1], event.data[2], event.data[3])
    } else {
        return None;
    };

    let amount_int = felt_to_biguint(amount_low) + (felt_to_biguint(amount_high) << 128usize);
    let amount = BigDecimal::from_biguint(amount_int, 6);
    Some(ParsedTransfer {
        from: felt_hex(from),
        to: felt_hex(to),
        amount,
    })
}

async fn active_merchants(pool: &DbPool) -> Result<Vec<Merchant>> {
    let mut conn = pool.get().await?;
    Ok(merchants::table
        .filter(merchants::is_active.eq(true))
        .load::<Merchant>(&mut conn)
        .await?)
}

async fn process_contract(
    pool: &DbPool,
    provider: &JsonRpcClient<HttpTransport>,
    contract_address: &str,
    currency: &str,
    from_block: u64,
    to_block: u64,
    address_set: &HashMap<String, String>,
) -> Result<()> {
    let selector = get_selector_from_name("Transfer").map_err(|err| anyhow!(err.to_string()))?;
    let contract = Felt::from_hex(contract_address).map_err(|err| anyhow!(err.to_string()))?;
    let mut continuation_token = None;

    loop {
        let page = provider
            .get_events(
                EventFilter {
                    from_block: Some(BlockId::Number(from_block)),
                    to_block: Some(BlockId::Number(to_block)),
                    address: Some(contract),
                    keys: Some(vec![vec![selector]]),
                },
                continuation_token,
                100,
            )
            .await?;

        for event in page.events {
            let Some(parsed) = parse_transfer_event(&event) else {
                continue;
            };
            if is_zero_addr(&parsed.from) {
                continue;
            }
            let Some(merchant_id) = address_set.get(&normalize_addr(&parsed.to)).cloned() else {
                continue;
            };

            insert_transaction(pool, currency, event, parsed, merchant_id).await?;
        }

        continuation_token = page.continuation_token;
        if continuation_token.is_none() {
            break;
        }
    }

    Ok(())
}

async fn insert_transaction(
    pool: &DbPool,
    currency: &str,
    event: EmittedEvent,
    parsed: ParsedTransfer,
    merchant_id: String,
) -> Result<()> {
    let tx_hash = felt_hex(event.transaction_hash);
    let mut conn = pool.get().await?;
    let existing = transactions::table
        .filter(transactions::tx_hash.eq(&tx_hash))
        .select(transactions::id)
        .first::<String>(&mut conn)
        .await
        .optional()?;
    if existing.is_some() {
        return Ok(());
    }

    let now = chrono::Utc::now();
    let tx = Transaction {
        id: Uuid::new_v4().to_string(),
        merchant_id: merchant_id.clone(),
        tx_hash,
        from_address: parsed.from,
        to_address: parsed.to,
        amount: parsed.amount,
        currency: currency.to_string(),
        status: "confirmed".to_string(),
        block_number: Some(event.block_number.unwrap_or_default().to_string()),
        note: None,
        email_sent: false,
        detected_at: now,
        confirmed_at: Some(now),
    };

    let new_tx = diesel::insert_into(transactions::table)
        .values(&tx)
        .get_result::<Transaction>(&mut conn)
        .await?;
    let merchant = merchants::table
        .find(merchant_id)
        .get_result::<Merchant>(&mut conn)
        .await
        .optional()?;
    drop(conn);

    if let Some(merchant) = merchant {
        let fiat_amount = match (
            new_tx.amount.to_f64(),
            convert(1.0, &merchant.currency).await.ok().flatten(),
        ) {
            (Some(amount), Some(fx)) => Some(amount * fx.rate),
            _ => None,
        };

        if send_payment_received(&merchant, &new_tx, fiat_amount)
            .await
            .is_ok()
        {
            let mut conn = pool.get().await?;
            diesel::update(transactions::table.find(&new_tx.id))
                .set(transactions::email_sent.eq(true))
                .execute(&mut conn)
                .await?;
        }
    }

    Ok(())
}
