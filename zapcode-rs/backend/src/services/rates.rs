use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct RateResult {
    pub rate: f64,
    pub fetched_at: i64,
    pub sources: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertedAmount {
    pub amount: f64,
    pub rate: f64,
    pub currency: String,
    pub fetched_at: i64,
    pub age_seconds: i64,
    pub sources: Vec<String>,
}

#[derive(Deserialize)]
struct BinancePrice {
    price: String,
}

async fn from_binance(to: &str) -> Result<f64> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;
    let usdc_res = client
        .get("https://data-api.binance.vision/api/v3/ticker/price?symbol=USDCUSDT")
        .send()
        .await
        .context("Binance USDCUSDT request failed")?;
    if !usdc_res.status().is_success() {
        return Err(anyhow!("Binance USDCUSDT returned {}", usdc_res.status()));
    }
    let usdc_data: BinancePrice = usdc_res.json().await?;
    let usdc_usd_rate: f64 = usdc_data.price.parse()?;

    let fiat_res = client
        .get(&format!(
            "https://data-api.binance.vision/api/v3/ticker/price?symbol=USDT{}",
            to.to_uppercase()
        ))
        .send()
        .await
        .context("Binance USDT pair request failed")?;
    if !fiat_res.status().is_success() {
        return Err(anyhow!(
            "Binance USDT{} returned {}",
            to.to_uppercase(),
            fiat_res.status()
        ));
    }
    let fiat_data: BinancePrice = fiat_res.json().await?;
    let usdt_fiat_rate: f64 = fiat_data.price.parse()?;

    Ok(usdc_usd_rate * usdt_fiat_rate)
}

async fn from_coingecko(to: &str) -> Result<f64> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;
    let url = format!(
        "https://api.coingecko.com/api/v3/simple/price?ids=usd-coin&vs_currencies={}",
        to.to_lowercase()
    );
    let res = client.get(&url).send().await?;
    if !res.status().is_success() {
        return Err(anyhow!("CoinGecko returned {}", res.status()));
    }
    let data: serde_json::Value = res.json().await?;
    let rate = data
        .get("usd-coin")
        .and_then(|v| v.get(&to.to_lowercase()))
        .and_then(|v| v.as_f64())
        .ok_or_else(|| anyhow!("CoinGecko: no rate for {}", to))?;
    Ok(rate)
}

async fn from_exchange_rate_api(to: &str) -> Result<f64> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;
    let res = client
        .get("https://open.er-api.com/v6/latest/USD")
        .send()
        .await?;
    if !res.status().is_success() {
        return Err(anyhow!("ExchangeRateAPI returned {}", res.status()));
    }
    let data: serde_json::Value = res.json().await?;
    let rate = data
        .get("rates")
        .and_then(|rates| rates.get(&to.to_uppercase()))
        .and_then(|v| v.as_f64())
        .ok_or_else(|| anyhow!("ExchangeRateAPI: no rate for {}", to))?;
    Ok(rate)
}

pub async fn fetch_rate(_from: &str, to: &str) -> Result<RateResult> {
    let mut successful = Vec::new();
    let mut errors = Vec::new();

    let (binance, coingecko, exchange) = tokio::join!(
        from_binance(to),
        from_coingecko(to),
        from_exchange_rate_api(to),
    );

    if let Ok(rate) = binance {
        successful.push(("Binance".to_string(), rate));
    } else if let Err(err) = binance {
        errors.push(err.to_string());
    }
    if let Ok(rate) = coingecko {
        successful.push(("CoinGecko".to_string(), rate));
    } else if let Err(err) = coingecko {
        errors.push(err.to_string());
    }
    if let Ok(rate) = exchange {
        successful.push(("ExchangeRateAPI".to_string(), rate));
    } else if let Err(err) = exchange {
        errors.push(err.to_string());
    }

    if successful.is_empty() {
        return Err(anyhow!("All rate oracles failed: {}", errors.join("; ")));
    }

    let rate = successful.iter().map(|(_, r)| *r).sum::<f64>() / successful.len() as f64;
    let fetched_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
    let sources = successful.into_iter().map(|(source, _)| source).collect();

    Ok(RateResult {
        rate,
        fetched_at,
        sources,
    })
}

pub async fn convert(usdc_amount: f64, to_currency: &str) -> Result<Option<ConvertedAmount>> {
    let rate_result = fetch_rate("USDC", to_currency).await;
    match rate_result {
        Ok(rate_result) => Ok(Some(ConvertedAmount {
            amount: usdc_amount * rate_result.rate,
            rate: rate_result.rate,
            currency: to_currency.to_uppercase(),
            fetched_at: rate_result.fetched_at,
            age_seconds: (std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_secs() as i64)
                - rate_result.fetched_at,
            sources: rate_result.sources,
        })),
        Err(_) => Ok(None),
    }
}
