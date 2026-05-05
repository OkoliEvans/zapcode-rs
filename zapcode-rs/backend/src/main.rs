use axum::{response::Json, routing::get, Router};
use dotenvy::dotenv;
use serde_json::json;
use std::env;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use zapcode_rs::{db, routes};

#[tokio::main]
async fn main() {
    dotenvy::from_path(concat!(env!("CARGO_MANIFEST_DIR"), "/.env")).ok();
    dotenv().ok();

    // Initialize database pool
    let pool = db::create_db_pool()
        .await
        .expect("Failed to create database pool");

    // Build the application
    let app = Router::new()
        .route("/health", get(health))
        .nest("/api/merchants", routes::merchants::router())
        .nest("/api/transactions", routes::transactions::router())
        .nest("/api/wallet", routes::wallet::router())
        .nest("/api/rates", routes::rates::router())
        .nest("/api/stats", routes::stats::router())
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()))
        .with_state(pool);

    // Get port from environment
    let port = env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let addr = format!("{}:{}", host, port);

    println!("🚀 Server running on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "ok": true,
        "ts": chrono::Utc::now().to_rfc3339()
    }))
}
