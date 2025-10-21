use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

/// Connected client info
pub struct Client {
    pub user_id: Uuid,
    pub username: String,
    pub joint_id: Uuid,
    pub sender: mpsc::UnboundedSender<String>,
}

/// Manages all WebSocket connections
#[derive(Clone)]
pub struct ConnectionManager {
    // Map: user_id -> Client
    clients: Arc<RwLock<HashMap<Uuid, Client>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a new client connection
    pub async fn add_client(
        &self,
        user_id: Uuid,
        username: String,
        joint_id: Uuid,
        sender: mpsc::UnboundedSender<String>,
    ) {
        let client = Client {
            user_id,
            username: username.clone(),
            joint_id,
            sender,
        };

        self.clients.write().await.insert(user_id, client);
        tracing::info!("User {} connected to joint {}", username, joint_id);
    }

    /// Remove a client connection
    pub async fn remove_client(&self, user_id: &Uuid) {
        if let Some(client) = self.clients.write().await.remove(user_id) {
            tracing::info!(
                "User {} disconnected from joint {}",
                client.username,
                client.joint_id
            );
        }
    }

    /// Broadcast message to all users in a joint
    pub async fn broadcast_to_joint(&self, joint_id: Uuid, message: String, sender_id: Uuid) {
        let clients = self.clients.read().await;

        for (user_id, client) in clients.iter() {
            // Send to all users in the joint EXCEPT the sender
            if client.joint_id == joint_id && *user_id != sender_id {
                if let Err(e) = client.sender.send(message.clone()) {
                    tracing::error!("Failed to send message to user {}: {}", user_id, e);
                }
            }
        }
    }

    /// Get count of online users in a joint
    pub async fn get_joint_user_count(&self, joint_id: Uuid) -> usize {
        self.clients
            .read()
            .await
            .values()
            .filter(|c| c.joint_id == joint_id)
            .count()
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}
