use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// User from database
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub profile_pic_url: Option<String>,
    pub phone: Option<String>,
    pub is_18_plus: bool,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Registration request
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,

    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8))]
    pub password: String,

    pub is_18_plus: bool,
}

/// Registration response
#[derive(Debug, Serialize, ToSchema)]
pub struct RegisterResponse {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub message: String,
}

/// Login request
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(length(min = 1))]
    pub username: String,

    #[validate(length(min = 1))]
    pub password: String,
}

/// Login response
#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// OTP verification request
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct VerifyOtpRequest {
    #[validate(email)]
    pub email: String,

    #[validate(length(equal = 6))]
    pub code: String,
}

/// OTP verification response
#[derive(Debug, Serialize, ToSchema)]
pub struct VerifyOtpResponse {
    pub message: String,
    pub verified: bool,
}
