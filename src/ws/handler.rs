use axum::{
    Extension,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::ws::manager::ConnectionManager;

/// Incoming message from client
#[derive(Debug, Deserialize)]
struct IncomingMessage {
    content: String,
}

/// Outgoing message to client
#[derive(Debug, Serialize)]
struct OutgoingMessage {
    id: Uuid,
    joint_id: Uuid,
    user_id: Uuid,
    username: String,
    content: String,
    created_at: String,
}

/// Handle WebSocket upgrade
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<crate::models::app_state::AppState>,
    Extension(user_id): Extension<Uuid>,
    Extension(joint_id): Extension<Uuid>,
    Extension(username): Extension<String>,
) -> Response {
    let manager = state.ws_manager.clone();
    let db = state.db.clone();
    ws.on_upgrade(move |socket| handle_socket(socket, manager, db, user_id, joint_id, username))
}

/// Handle individual WebSocket connection
async fn handle_socket(
    socket: WebSocket,
    manager: ConnectionManager,
    db: PgPool,
    user_id: Uuid,
    joint_id: Uuid,
    username: String,
) {
    let (mut sender, mut receiver) = socket.split();

    // Create channel for this client
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    // Register client in manager
    manager
        .add_client(user_id, username.clone(), joint_id, tx)
        .await;

    // Task to send messages TO client
    let mut send_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if sender.send(Message::Text(message.into())).await.is_err() {
                break;
            }
        }
    });

    // Task to receive messages FROM client
    let manager_clone = manager.clone();
    let username_clone = username.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(message)) = receiver.next().await {
            if let Message::Text(text) = message {
                // Parse incoming message
                if let Ok(incoming) = serde_json::from_str::<IncomingMessage>(&text) {
                    // Save message to database
                    let message_id = Uuid::new_v4();
                    let created_at = chrono::Utc::now();

                    if let Err(e) = sqlx::query!(
                        r#"
                        INSERT INTO messages (id, joint_id, user_id, content, created_at)
                        VALUES ($1, $2, $3, $4, $5)
                        "#,
                        message_id,
                        joint_id,
                        user_id,
                        incoming.content,
                        created_at
                    )
                    .execute(&db)
                    .await
                    {
                        tracing::error!("Failed to save message: {}", e);
                        continue;
                    }

                    // Create outgoing message
                    let outgoing = OutgoingMessage {
                        id: message_id,
                        joint_id,
                        user_id,
                        username: username_clone.clone(),
                        content: incoming.content,
                        created_at: created_at.to_rfc3339(),
                    };

                    // Broadcast to all users in joint
                    if let Ok(json) = serde_json::to_string(&outgoing) {
                        manager_clone
                            .broadcast_to_joint(joint_id, json, user_id)
                            .await;
                    }
                }
            } else if let Message::Close(_) = message {
                break;
            }
        }
    });

    // Wait for either task to finish (means disconnect)
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    // Cleanup: remove client from manager
    manager.remove_client(&user_id).await;
    tracing::info!("User {} disconnected from joint {}", username, joint_id);
}
