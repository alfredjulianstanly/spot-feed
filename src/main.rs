mod api;
mod errors;
mod middleware;
mod models;
mod utils;

use axum::{Json, Router, extract::State, middleware as axum_middleware, routing::get};

use serde_json::{Value, json};
use shuttle_axum::ShuttleAxum;
use sqlx::postgres::PgPoolOptions;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::auth::{login, register, verify_otp};
use crate::api::joints::{
    create_joint, get_active_joints, join_joint, leave_joint, list_nearby_joints,
};
use crate::api::profile::{get_profile, update_profile};
use crate::middleware::auth::auth_middleware;
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
        crate::api::joints::get_active_joints,      
        crate::api::joints::leave_joint, 
        crate::api::profile::get_profile,
        crate::api::profile::update_profile,         
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
            crate::models::profile::UserProfile,         
            crate::models::profile::UpdateProfileRequest, 
            crate::models::profile::ProfileResponse,
        )
    ),
    tags(
        (name = "Authentication", description = "User authentication endpoints"),
        (name = "Joints", description = "Location-based group endpoints"),
        (name = "Profile", description = "User profile management")
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
async fn main(
    #[shuttle_shared_db::Postgres] conn_str: String,
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> ShuttleAxum {
    // Get Resend API key from secrets
    let resend_api_key = secrets
        .get("RESEND_API_KEY")
        .expect("RESEND_API_KEY must be set in Secrets.toml");

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&conn_str)
        .await
        .expect("Failed to connect to database");

    let state = AppState::new(db, resend_api_key);

    // Protected routes that require authentication
    let protected_routes = Router::new()
        .route("/api/v1/joints", axum::routing::post(create_joint))
        .route(
            "/api/v1/joints/active",
            axum::routing::get(get_active_joints),
        ) // ADD
        .route("/api/v1/joints/join", axum::routing::post(join_joint))
        .route("/api/v1/joints/leave", axum::routing::post(leave_joint)) 
        .route("/api/v1/profile", axum::routing::get(get_profile)) 
        .route("/api/v1/profile", axum::routing::put(update_profile)) 
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Public routes
    let public_routes = Router::new()
        .route("/", get(hello_world))
        .route("/api/health", get(health_check))
        // Authentication routes
        .route("/api/v1/auth/register", axum::routing::post(register))
        .route("/api/v1/auth/verify-otp", axum::routing::post(verify_otp))
        .route("/api/v1/auth/login", axum::routing::post(login))
        // Public joints routes
        .route(
            "/api/v1/joints/nearby",
            axum::routing::post(list_nearby_joints),
        );

    let router = Router::new()
        .merge(protected_routes)
        .merge(public_routes)
        // Swagger UI
        .merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", ApiDoc::openapi()))
        .with_state(state);

    Ok(router.into())
}
