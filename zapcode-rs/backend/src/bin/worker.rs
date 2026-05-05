use dotenvy::dotenv;
use std::time::Duration;
use zapcode_rs::{db, services::watcher::Watcher};

#[tokio::main]
async fn main() {
    dotenvy::from_path(concat!(env!("CARGO_MANIFEST_DIR"), "/.env")).ok();
    dotenv().ok();

    let pool = db::create_db_pool()
        .await
        .expect("Failed to create database pool");
    let interval_ms = std::env::var("POLL_INTERVAL_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(3_000);

    println!(
        "[worker] USDC watcher starting - interval {}ms",
        interval_ms
    );
    println!(
        "[worker] network: {}",
        std::env::var("STARKNET_NETWORK").unwrap_or_else(|_| "sepolia".to_string())
    );

    let mut watcher = Watcher::new();
    let mut interval = tokio::time::interval(Duration::from_millis(interval_ms));

    loop {
        interval.tick().await;
        if let Err(err) = watcher.poll_once(&pool).await {
            eprintln!("[worker] poll error: {}", err);
        }
    }
}
