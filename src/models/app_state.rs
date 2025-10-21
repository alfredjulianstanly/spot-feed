use crate::ws::manager::ConnectionManager;
use sqlx::PgPool;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub resend_api_key: String,
    pub ws_manager: ConnectionManager,
}

impl AppState {
    pub fn new(db: PgPool, resend_api_key: String) -> Self {
        Self {
            db,
            resend_api_key,
            ws_manager: ConnectionManager::new(),
        }
    }
}
