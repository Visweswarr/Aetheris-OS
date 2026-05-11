//! IPC communication module for AI Core Service

use std::path::Path;
use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use tokio::sync::{RwLock, mpsc};



use crate::model::ModelManager;
use crate::tools::ToolRegistry;
use crate::cap::CapTokenManager;
use crate::router::PromptRouter;
use crate::memory::MemoryStore;
use crate::notifications::NotificationManager;
use crate::error::Result;

pub use crate::generated::ai_core::*;

#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};

pub struct IpcServer {
    socket_path: std::path::PathBuf,
    model_manager: Arc<ModelManager>,
    tool_registry: Arc<ToolRegistry>,
    cap_token_manager: Arc<CapTokenManager>,
    prompt_router: Arc<PromptRouter>,
    memory_store: Arc<MemoryStore>,
    notification_manager: Arc<NotificationManager>,
    max_sessions: usize,
    request_timeout: Duration,
    active_sessions: Arc<RwLock<HashMap<String, SessionInfo>>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

#[derive(Debug, Clone)]
struct SessionInfo {
    session_id: String,
    client_id: String,
    connected_at: SystemTime,
    last_activity: SystemTime,
    request_count: u64,
}

impl IpcServer {
    pub async fn new(
        socket_path: &Path,
        model_manager: Arc<ModelManager>,
        tool_registry: Arc<ToolRegistry>,
        cap_token_manager: Arc<CapTokenManager>,
        prompt_router: Arc<PromptRouter>,
        memory_store: Arc<MemoryStore>,
        notification_manager: Arc<NotificationManager>,
        max_sessions: usize,
        request_timeout_secs: u64,
    ) -> Result<Self> {
        if let Some(parent) = socket_path.parent() {
            tokio::fs::create_dir_all(parent).await.ok();
        }
        if socket_path.exists() {
            tokio::fs::remove_file(socket_path).await.ok();
        }
        Ok(Self {
            socket_path: socket_path.to_path_buf(),
            model_manager, tool_registry, cap_token_manager, prompt_router,
            memory_store, notification_manager, max_sessions,
            request_timeout: Duration::from_secs(request_timeout_secs),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            shutdown_tx: None,
        })
    }

    pub async fn start(&self) -> Result<()> {
        // Starting IPC server
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        // Stopping IPC server
        Ok(())
    }
}

impl Clone for IpcServer {
    fn clone(&self) -> Self {
        Self {
            socket_path: self.socket_path.clone(),
            model_manager: self.model_manager.clone(),
            tool_registry: self.tool_registry.clone(),
            cap_token_manager: self.cap_token_manager.clone(),
            prompt_router: self.prompt_router.clone(),
            memory_store: self.memory_store.clone(),
            notification_manager: self.notification_manager.clone(),
            max_sessions: self.max_sessions,
            request_timeout: self.request_timeout,
            active_sessions: self.active_sessions.clone(),
            shutdown_tx: None,
        }
    }
}
