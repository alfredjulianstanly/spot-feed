mod api;
mod errors;
mod models;
mod utils;

use axum::{extract::State, routing::get, Json, Router};
use serde_json::{json, Value};
use shuttle_axum::ShuttleAxum;
use sqlx::postgres::PgPoolOptions;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::auth::{login, register, verify_otp};
use crate::models::app_state::AppState;

/// API Documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::auth::register,
        crate::api::auth::verify_otp,
        crate::api::auth::login,
    ),
    components(
        schemas(
            crate::models::user::RegisterRequest,
            crate::models::user::RegisterResponse,
            crate::models::user::VerifyOtpRequest,
            crate::models::user::VerifyOtpResponse,
            crate::models::user::LoginRequest,
            crate::models::user::LoginResponse,
        )
    ),
    tags(
        (name = "Authentication", description = "User authentication endpoints")
    ),
    info(
        title = "Spot Feed API",
        version = "0.1.0",
        description = "Location-based social networking API",
        contact(
            name = "Spot Feed Team",
            email = "support@spotfeed.com"
        )
    )
)]
struct ApiDoc;

async fn hello_world() -> &'static str {
    "Hello from Spot Feed! 🚀"
}

async fn health_check(State(state): State<AppState>) -> Json<Value> {
    // Test database connection
    match sqlx::query("SELECT 1 as health_check")
        .fetch_one(&state.db)
        .await
    {
        Ok(_) => Json(json!({
            "status": "healthy",
            "database": "connected"
        })),
        Err(e) => Json(json!({
            "status": "unhealthy",
            "database": "disconnected",
            "error": e.to_string()
        })),
    }
}

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] conn_str: String) -> ShuttleAxum {
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&conn_str)
        .await
        .expect("Failed to connect to database");

    let state = AppState::new(db);

    let router = Router::new()
        .route("/", get(hello_world))
        .route("/api/health", get(health_check))
        // Authentication routes
        .route("/api/v1/auth/register", axum::routing::post(register))
        .route("/api/v1/auth/verify-otp", axum::routing::post(verify_otp))
        .route("/api/v1/auth/login", axum::routing::post(login))
        // Swagger UI
        .merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", ApiDoc::openapi()))
        .with_state(state);

    Ok(router.into())
}
