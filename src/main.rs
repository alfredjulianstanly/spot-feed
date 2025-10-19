use axum::{extract::State, routing::get, Json, Router};
use serde_json::{json, Value};
use shuttle_axum::ShuttleAxum;
use sqlx::{postgres::PgPoolOptions, PgPool};

mod errors;

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

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

    let state = AppState { db };

    let router = Router::new()
        .route("/", get(hello_world))
        .route("/health", get(health_check))
        .with_state(state);

    Ok(router.into())
}
