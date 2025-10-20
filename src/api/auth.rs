use axum::{extract::State, http::StatusCode, Json};
use rand::Rng;
use validator::Validate;

use crate::{
    errors::AppError,
    models::{
        app_state::AppState,
        user::{
            LoginRequest, LoginResponse, RegisterRequest, RegisterResponse, VerifyOtpRequest,
            VerifyOtpResponse,
        },
    },
    utils::password::hash_password,
};

/// Register a new user
#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = RegisterResponse),
        (status = 400, description = "Invalid input or user already exists"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Authentication"
)]
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), AppError> {
    // Validate input
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // Check if user is 18+
    if !payload.is_18_plus {
        return Err(AppError::ValidationError(
            "You must be 18 or older to register".to_string(),
        ));
    }

    // Check if username already exists
    let existing_user = sqlx::query!(
        "SELECT id FROM users WHERE username = $1 OR email = $2",
        payload.username,
        payload.email
    )
    .fetch_optional(&state.db)
    .await?;

    if existing_user.is_some() {
        return Err(AppError::UserAlreadyExists);
    }

    // Hash password
    let password_hash = hash_password(&payload.password)?;

    // Insert user
    let user = sqlx::query!(
        r#"
        INSERT INTO users (username, email, password_hash, is_18_plus)
        VALUES ($1, $2, $3, $4)
        RETURNING id, username, email
        "#,
        payload.username,
        payload.email,
        password_hash,
        payload.is_18_plus
    )
    .fetch_one(&state.db)
    .await?;

    // Generate OTP code (6 digits)
    let otp_code = rand::rng().random_range(100000..999999).to_string();

    // Calculate OTP expiry (10 minutes from now)
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(10);

    // Save OTP to database
    sqlx::query!(
        r#"
        INSERT INTO otp_codes (user_id, code, expires_at)
        VALUES ($1, $2, $3)
        "#,
        user.id,
        otp_code,
        expires_at
    )
    .execute(&state.db)
    .await?;

    // Send OTP via email
    crate::utils::email::send_otp_email(&payload.email, &otp_code, &state.resend_api_key).await?;

    tracing::info!("OTP sent to {}", payload.email);

    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse {
            user_id: user.id,
            username: user.username,
            email: user.email,
            message: "Registration successful! Please check your email for OTP verification code."
                .to_string(),
        }),
    ))
}

/// Verify OTP code
#[utoipa::path(
    post,
    path = "/api/v1/auth/verify-otp",
    request_body = VerifyOtpRequest,
    responses(
        (status = 200, description = "Email verified successfully", body = VerifyOtpResponse),
        (status = 400, description = "Invalid or expired OTP"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Authentication"
)]
pub async fn verify_otp(
    State(state): State<AppState>,
    Json(payload): Json<VerifyOtpRequest>,
) -> Result<Json<VerifyOtpResponse>, AppError> {
    // Validate input
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // Find user by email
    let user = sqlx::query!("SELECT id FROM users WHERE email = $1", payload.email)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::UserNotFound)?;

    // Find valid OTP
    let otp = sqlx::query!(
        r#"
        SELECT id, code, expires_at, is_used
        FROM otp_codes
        WHERE user_id = $1 AND code = $2 AND is_used = false
        ORDER BY created_at DESC
        LIMIT 1
        "#,
        user.id,
        payload.code
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::InvalidOtp)?;

    // Check if OTP expired
    if otp.expires_at < chrono::Utc::now() {
        return Err(AppError::OtpExpired);
    }

    // Mark OTP as used
    sqlx::query!("UPDATE otp_codes SET is_used = true WHERE id = $1", otp.id)
        .execute(&state.db)
        .await?;

    // Mark user as verified
    sqlx::query!("UPDATE users SET is_verified = true WHERE id = $1", user.id)
        .execute(&state.db)
        .await?;

    Ok(Json(VerifyOtpResponse {
        message: "Email verified successfully!".to_string(),
        verified: true,
    }))
}

/// User login
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 400, description = "Email not verified"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Authentication"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    // Validate input
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // Find user by username
    let user = sqlx::query!(
        "SELECT id, password_hash, is_verified FROM users WHERE username = $1",
        payload.username
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::InvalidCredentials)?;

    // Check if user is verified
    if !user.is_verified.unwrap_or(false) {
        return Err(AppError::ValidationError(
            "Please verify your email before logging in".to_string(),
        ));
    }

    // Verify password
    let is_valid = crate::utils::password::verify_password(&payload.password, &user.password_hash)?;

    if !is_valid {
        return Err(AppError::InvalidCredentials);
    }

    // Generate JWT token
    // TODO: Get JWT secret and expiry from config
    let jwt_secret = "your-super-secret-jwt-key-change-in-production"; // Temporary
    let jwt_expiry_hours = 24;

    let token = crate::utils::jwt::generate_token(user.id, jwt_secret, jwt_expiry_hours)?;

    Ok(Json(LoginResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in: jwt_expiry_hours * 3600, // Convert to seconds
    }))
}
