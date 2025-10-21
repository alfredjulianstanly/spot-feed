// src/middleware/auth.rs
use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::{errors::AppError, models::app_state::AppState, utils::jwt::verify_token};

/// Extract and verify JWT token from Authorization header
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Get Authorization header
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    // Check if it starts with "Bearer "
    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::Unauthorized);
    }

    // Extract token
    let token = auth_header.trim_start_matches("Bearer ");

    // Verify token
    // TODO: Get JWT secret from config
    let jwt_secret = "your-super-secret-jwt-key-change-in-production";

    let claims = verify_token(token, jwt_secret)?;

    // Parse user_id from claims.sub
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;

    // Add user_id to request extensions
    req.extensions_mut().insert(user_id);

    Ok(next.run(req).await)
}

/// Optional auth - doesn't fail if no token, just doesn't add user_id
pub async fn optional_auth_middleware(
    State(_state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    // Try to get Authorization header
    if let Some(auth_header) = req.headers().get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = auth_str.trim_start_matches("Bearer ");
                let jwt_secret = "your-super-secret-jwt-key-change-in-production";

                // Try to verify token and parse user_id
                if let Ok(claims) = verify_token(token, jwt_secret) {
                    if let Ok(user_id) = Uuid::parse_str(&claims.sub) {
                        req.extensions_mut().insert(user_id);
                    }
                }
            }
        }
    }

    next.run(req).await
}
