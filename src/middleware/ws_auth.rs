use axum::{
    extract::{Path, Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::{errors::AppError, models::app_state::AppState, utils::jwt::verify_token};

/// WebSocket authentication middleware - extracts user info and joint_id
pub async fn ws_auth_middleware(
    State(state): State<AppState>,
    Path(joint_id): Path<Uuid>,
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
    let jwt_secret = "your-super-secret-jwt-key-change-in-production";
    let claims = verify_token(token, jwt_secret)?;

    // Parse user_id from claims
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;

    // Get username from database
    let user = sqlx::query!("SELECT username FROM users WHERE id = $1", user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    // Verify user is a member of the joint
    let is_member = sqlx::query!(
        "SELECT id FROM joint_members WHERE joint_id = $1 AND user_id = $2",
        joint_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await?;

    if is_member.is_none() {
        return Err(AppError::ValidationError(
            "You are not a member of this joint".to_string(),
        ));
    }

    // Add user_id, joint_id, and username to extensions
    req.extensions_mut().insert(user_id);
    req.extensions_mut().insert(joint_id);
    req.extensions_mut().insert(user.username);

    Ok(next.run(req).await)
}
