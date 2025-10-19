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

/// Register a new user account
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    /// Username (3-50 characters)
    #[validate(length(min = 3, max = 50))]
    #[schema(example = "johndoe")]
    pub username: String,

    /// Valid email address
    #[validate(email)]
    #[schema(example = "john@example.com")]
    pub email: String,

    /// Password (minimum 8 characters)
    #[validate(length(min = 8))]
    #[schema(example = "SecurePass123!")]
    pub password: String,

    /// Must be 18 years or older
    #[schema(example = true)]
    pub is_18_plus: bool,
}

/// Successful registration response
#[derive(Debug, Serialize, ToSchema)]
pub struct RegisterResponse {
    /// Unique user identifier
    pub user_id: Uuid,
    /// Username
    pub username: String,
    /// Email address
    pub email: String,
    /// Success message
    pub message: String,
}

/// Login with username and password
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    /// Your username
    #[validate(length(min = 1))]
    #[schema(example = "johndoe")]
    pub username: String,

    /// Your password
    #[validate(length(min = 1))]
    #[schema(example = "SecurePass123!")]
    pub password: String,
}

/// Successful login response with JWT token
#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    /// JWT access token
    pub access_token: String,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Token expiration time in seconds
    pub expires_in: i64,
}

/// Verify OTP code sent to email
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct VerifyOtpRequest {
    /// Email address used during registration
    #[validate(email)]
    #[schema(example = "john@example.com")]
    pub email: String,

    /// 6-digit OTP code
    #[validate(length(equal = 6))]
    #[schema(example = "123456")]
    pub code: String,
}

/// OTP verification result
#[derive(Debug, Serialize, ToSchema)]
pub struct VerifyOtpResponse {
    /// Result message
    pub message: String,
    /// Whether verification was successful
    pub verified: bool,
}
