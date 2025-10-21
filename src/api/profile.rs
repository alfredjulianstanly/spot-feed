use axum::{Extension, Json, extract::State};
use uuid::Uuid;
use validator::Validate;

use crate::{
    errors::AppError,
    models::{
        app_state::AppState,
        profile::{ProfileResponse, UpdateProfileRequest, UserProfile},
    },
};

/// Get current user's profile
#[utoipa::path(
    get,
    path = "/api/v1/profile",
    responses(
        (status = 200, description = "User profile", body = ProfileResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Profile",
    security(("bearer" = []))
)]
pub async fn get_profile(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<ProfileResponse>, AppError> {
    let profile = sqlx::query_as!(
        UserProfile,
        r#"
        SELECT id, username, email, display_name, profile_picture_url, phone_number, created_at
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(ProfileResponse { profile }))
}

/// Update current user's profile
#[utoipa::path(
    put,
    path = "/api/v1/profile",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated successfully", body = ProfileResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Profile",
    security(("bearer" = []))
)]
pub async fn update_profile(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<ProfileResponse>, AppError> {
    // Validate input
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // Update profile
    let profile = sqlx::query_as!(
        UserProfile,
        r#"
        UPDATE users
        SET 
            display_name = COALESCE($1, display_name),
            profile_picture_url = COALESCE($2, profile_picture_url),
            phone_number = COALESCE($3, phone_number)
        WHERE id = $4
        RETURNING id, username, email, display_name, profile_picture_url, phone_number, created_at
        "#,
        payload.display_name,
        payload.profile_picture_url,
        payload.phone_number,
        user_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(ProfileResponse { profile }))
}
