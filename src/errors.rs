use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Application-wide error type
#[derive(Debug)]
pub enum AppError {
    // Database errors
    DatabaseError(sqlx::Error),

    // Authentication errors
    InvalidCredentials,
    InvalidToken,
    TokenExpired,
    Unauthorized,

    // Validation errors
    ValidationError(String),

    // User errors
    UserAlreadyExists,
    UserNotFound,

    // OTP errors
    InvalidOtp,
    OtpExpired,

    // Internal errors
    InternalError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::DatabaseError(e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error occurred".to_string(),
                )
            }
            AppError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "Invalid username or password".to_string(),
            ),
            AppError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token".to_string()),
            AppError::TokenExpired => (StatusCode::UNAUTHORIZED, "Token expired".to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::ValidationError(msg) => {
                (StatusCode::BAD_REQUEST, msg) // Already a String!
            }
            AppError::UserAlreadyExists => {
                (StatusCode::CONFLICT, "User already exists".to_string())
            }
            AppError::UserNotFound => (StatusCode::NOT_FOUND, "User not found".to_string()),
            AppError::InvalidOtp => (StatusCode::BAD_REQUEST, "Invalid OTP code".to_string()),
            AppError::OtpExpired => (StatusCode::BAD_REQUEST, "OTP code expired".to_string()),
            AppError::InternalError(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

// Implement From trait for automatic conversion
impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::DatabaseError(e)
    }
}
