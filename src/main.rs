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
use crate::api::joints::{create_joint, join_joint, list_nearby_joints};
use crate::models::app_state::AppState;

/// API Documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::auth::register,
        crate::api::auth::verify_otp,
        crate::api::auth::login,
        crate::api::joints::create_joint,           
        crate::api::joints::list_nearby_joints,   
        crate::api::joints::join_joint,            
    ),
    components(
        schemas(
            crate::models::user::RegisterRequest,
            crate::models::user::RegisterResponse,
            crate::models::user::VerifyOtpRequest,
            crate::models::user::VerifyOtpResponse,
            crate::models::user::LoginRequest,
            crate::models::user::LoginResponse,
            crate::models::joint::CreateJointRequest,      
            crate::models::joint::CreateJointResponse,    
            crate::models::joint::ListJointsRequest,     
            crate::models::joint::ListJointsResponse,   
            crate::models::joint::JoinJointRequest,    
            crate::models::joint::JoinJointResponse,  
            crate::models::joint::Joint,             
            crate::models::joint::JointWithDistance,
        )
    ),
    tags(
        (name = "Authentication", description = "User authentication endpoints"),
        (name = "Joints", description = "Location-based group endpoints")
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

        // Joints routes
        .route("/api/v1/joints", axum::routing::post(create_joint))
        .route("/api/v1/joints/nearby", axum::routing::post(list_nearby_joints))
        .route("/api/v1/joints/join", axum::routing::post(join_joint))

        // Swagger UI
        .merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", ApiDoc::openapi()))
        .with_state(state);

    Ok(router.into())
}
