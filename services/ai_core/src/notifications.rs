//! AI Core Notification Action Handler

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

use crate::error::{AiCoreError, Result};
use crate::tools::ToolRegistry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationActionRequest {
    pub notification_id: String,
    pub action_id: String,
    pub tool_name: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationActionResponse {
    pub notification_id: String,
    pub action_id: String,
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub timestamp_ms: u64,
}

pub struct NotificationManager {
    tool_registry: Arc<ToolRegistry>,
    active_notifications: Arc<RwLock<HashMap<String, ActiveNotification>>>,
}

fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[derive(Debug, Clone)]
struct ActiveNotification {
    pub id: String,
    pub title: String,
    pub message: Option<String>,
    pub notification_type: String,
    pub actions: Vec<NotificationAction>,
    pub created_at_ms: u64,
    pub expires_at_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    pub id: String,
    pub label: String,
    pub tool_name: Option<String>,
    pub parameters: Option<HashMap<String, serde_json::Value>>,
    pub style: Option<String>,
    pub icon: Option<String>,
}

impl NotificationManager {
    pub fn new(tool_registry: Arc<ToolRegistry>) -> Self {
        Self {
            tool_registry,
            active_notifications: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn handle_notification_action(
        &self,
        request: NotificationActionRequest,
    ) -> Result<NotificationActionResponse> {
        let notification = self.get_notification(&request.notification_id).await?;

        let _action = notification
            .actions
            .iter()
            .find(|a| a.id == request.action_id)
            .ok_or_else(|| {
                AiCoreError::NotFound(format!("Action {} not found", request.action_id))
            })?;

        let result = serde_json::Value::Bool(true);

        Ok(NotificationActionResponse {
            notification_id: request.notification_id,
            action_id: request.action_id,
            success: true,
            result: Some(result),
            error: None,
            timestamp_ms: current_timestamp_ms(),
        })
    }

    async fn get_notification(&self, notification_id: &str) -> Result<ActiveNotification> {
        self.active_notifications
            .read()
            .await
            .get(notification_id)
            .cloned()
            .ok_or_else(|| {
                AiCoreError::NotFound(format!("Notification {} not found", notification_id))
            })
    }

    pub async fn create_notification(
        &self,
        title: String,
        message: Option<String>,
        notification_type: String,
        actions: Vec<NotificationAction>,
    ) -> Result<String> {
        let notification_id = format!(
            "notif-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let notification = ActiveNotification {
            id: notification_id.clone(),
            title,
            message,
            notification_type,
            actions,
            created_at_ms: current_timestamp_ms(),
            expires_at_ms: None,
        };
        self.active_notifications
            .write()
            .await
            .insert(notification_id.clone(), notification);
        Ok(notification_id)
    }

    pub async fn dismiss_notification(&self, notification_id: &str) -> Result<()> {
        self.active_notifications
            .write()
            .await
            .remove(notification_id);
        Ok(())
    }

    pub async fn get_active_notifications(&self) -> Vec<String> {
        self.active_notifications
            .read()
            .await
            .keys()
            .cloned()
            .collect()
    }
}

pub fn create_notification_manager(tool_registry: Arc<ToolRegistry>) -> NotificationManager {
    NotificationManager::new(tool_registry)
}
