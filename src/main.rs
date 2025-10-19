use shuttle_axum::ShuttleAxum;
use axum::{routing::get, Router};

async fn hello_world() -> &'static str {
    "Hello from Spot Feed! 🚀"
}

#[shuttle_runtime::main]
async fn main() -> ShuttleAxum {
    let router = Router::new()
        .route("/", get(hello_world));

    Ok(router.into())
}
