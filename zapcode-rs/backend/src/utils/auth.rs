use axum::async_trait;
use axum::body::Body;
use axum::extract::FromRequestParts;
use axum::http::{header, request::Parts, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use axum::Json;
use serde_json::json;

use crate::services::privy::privy_client;

pub struct AuthUser(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "Missing auth token"})),
                )
            })?;

        let privy = privy_client().await.map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Privy is not configured"})),
            )
        })?;

        let user_id = privy
            .authenticate_user(auth_header)
            .await
            .map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "Invalid or expired token"})),
                )
            })?
            .user_id;

        Ok(AuthUser(user_id))
    }
}

pub async fn require_auth(
    mut request: axum::http::Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let privy = privy_client()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user_id = privy
        .authenticate_user(auth_header)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?
        .user_id;

    request.extensions_mut().insert(user_id);
    Ok(next.run(request).await)
}
