//! XR Scene Graph Service for Aetheris OS
//! 
//! This service provides a deterministic scene graph with DID-bound avatars,
//! capability-based access control, and NGFS snapshot integration.

pub mod scene;
pub mod avatar;
pub mod storage;
pub mod policy;
pub mod cap;
pub mod session;
pub mod dao_hook;
pub mod rpc;
pub mod error;

#[cfg(test)]
pub mod tests;

pub use scene::{SceneGraph, SceneNode, NodeId, Transform, Component, ComponentType};
pub use avatar::{Avatar, AvatarId, AvatarProfile, AvatarStatus};
pub use storage::{SceneSnapshot, SnapshotId, SnapshotManager, SnapshotUpdate, SnapshotStats};
pub use policy::{PolicyManager, PolicyBundle, PolicyRule, PolicyScope, PolicyAction, PolicyEffect, PolicyContext, PolicyResult, PolicyStats, AttachPolicyRequest, AttachPolicyResponse, SimulateRequest, SimulateResponse, OperationResult, SimulationOperation};
pub use cap::{CapManager, CapToken, CapScope, CapAction, Permission, CapTokenUpdate, CapCheckResult, CapabilityStats};
pub use session::{SessionManager, SessionId, SessionState, BeginSessionRequest, BeginSessionResponse, EndSessionRequest, EndSessionResponse, RefreshSessionRequest, RefreshSessionResponse, SessionValidation, DidProof};
pub use dao_hook::{DaoHook, DaoClient, DefaultDaoHook, MockDaoClient, DaoHookFactory, SceneProposal, ProposalStatus, ProposalVote, VoteChoice, DaoDecision, VoteSummary};
pub use rpc::{SceneRpcService, RpcRequest, RpcResponse, BeginSessionRpcRequest, BeginSessionRpcResponse, EndSessionRpcRequest, EndSessionRpcResponse, IssueCapRpcRequest, IssueCapRpcResponse, SpawnNodeRpcRequest, SpawnNodeRpcResponse, MoveNodeRpcRequest, MoveNodeRpcResponse, DeleteNodeRpcRequest, DeleteNodeRpcResponse, AttachPolicyRpcRequest, AttachPolicyRpcResponse, SimulateRpcRequest, SimulateRpcResponse, SnapshotSaveRpcRequest, SnapshotSaveRpcResponse, SnapshotLoadRpcRequest, SnapshotLoadRpcResponse, SceneEvent, AuditRecord};
pub use error::{SceneError, SceneResult};

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Scene service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneConfig {
    /// Maximum number of nodes in the scene
    pub max_nodes: usize,
    /// Maximum number of avatars
    pub max_avatars: usize,
    /// Tick rate in milliseconds
    pub tick_rate_ms: u64,
    /// Enable deterministic mode
    pub deterministic_mode: bool,
    /// Snapshot auto-save interval in seconds
    pub auto_save_interval: u64,
    /// Enable policy enforcement
    pub enable_policy_enforcement: bool,
    /// Enable capability checking
    pub enable_capability_checking: bool,
}

impl Default for SceneConfig {
    fn default() -> Self {
        Self {
            max_nodes: 10000,
            max_avatars: 1000,
            tick_rate_ms: 16, // ~60 FPS
            deterministic_mode: true,
            auto_save_interval: 300, // 5 minutes
            enable_policy_enforcement: true,
            enable_capability_checking: true,
        }
    }
}

/// Scene service statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneStats {
    pub total_nodes: usize,
    pub total_avatars: usize,
    pub active_nodes: usize,
    pub active_avatars: usize,
    pub snapshots_created: u64,
    pub snapshots_loaded: u64,
    pub policy_checks: u64,
    pub policy_denials: u64,
    pub capability_checks: u64,
    pub capability_denials: u64,
    pub last_tick_time: u64,
    pub average_tick_time: f64,
}

/// Main scene service
pub struct SceneService {
    config: SceneConfig,
    scene_graph: Arc<SceneGraph>,
    avatar_manager: Arc<avatar::AvatarManager>,
    snapshot_manager: Arc<SnapshotManager>,
    policy_manager: Arc<PolicyManager>,
    cap_manager: Arc<CapManager>,
    session_manager: Arc<SessionManager>,
    stats: Arc<RwLock<SceneStats>>,
    is_running: Arc<RwLock<bool>>,
}

impl SceneService {
    /// Create a new scene service
    pub async fn new(config: SceneConfig) -> SceneResult<Self> {
        let scene_graph = Arc::new(SceneGraph::new(config.max_nodes));
        let avatar_manager = Arc::new(avatar::AvatarManager::new(config.max_avatars));
        let snapshot_manager = Arc::new(SnapshotManager::new());
        let policy_manager = Arc::new(PolicyManager::new());
        let cap_manager = Arc::new(CapManager::new());
        let session_manager = Arc::new(SessionManager::default());

        let stats = Arc::new(RwLock::new(SceneStats {
            total_nodes: 0,
            total_avatars: 0,
            active_nodes: 0,
            active_avatars: 0,
            snapshots_created: 0,
            snapshots_loaded: 0,
            policy_checks: 0,
            policy_denials: 0,
            capability_checks: 0,
            capability_denials: 0,
            last_tick_time: 0,
            average_tick_time: 0.0,
        }));

        let is_running = Arc::new(RwLock::new(false));

        Ok(Self {
            config,
            scene_graph,
            avatar_manager,
            snapshot_manager,
            policy_manager,
            cap_manager,
            session_manager,
            stats,
            is_running,
        })
    }

    /// Start the scene service
    pub async fn start(&self) -> SceneResult<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(SceneError::ServiceAlreadyRunning);
        }
        *is_running = true;
        drop(is_running);

        // Start the tick loop
        self.start_tick_loop().await;

        Ok(())
    }

    /// Stop the scene service
    pub async fn stop(&self) -> SceneResult<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        Ok(())
    }

    /// Start the deterministic tick loop
    async fn start_tick_loop(&self) {
        let scene_graph = self.scene_graph.clone();
        let stats = self.stats.clone();
        let is_running = self.is_running.clone();
        let tick_rate = self.config.tick_rate_ms;

        tokio::spawn(async move {
            let mut tick_count = 0u64;
            let mut total_tick_time = 0u64;

            while *is_running.read().await {
                let start_time = std::time::Instant::now();

                // Perform scene graph tick
                scene_graph.tick().await;

                let tick_time = start_time.elapsed().as_millis() as u64;
                total_tick_time += tick_time;
                tick_count += 1;

                // Update statistics
                {
                    let mut stats = stats.write().await;
                    stats.last_tick_time = tick_time;
                    stats.average_tick_time = total_tick_time as f64 / tick_count as f64;
                }

                // Sleep for the remaining tick time
                if tick_time < tick_rate {
                    tokio::time::sleep(tokio::time::Duration::from_millis(tick_rate - tick_time)).await;
                }
            }
        });
    }

    /// Get the scene graph
    pub fn scene_graph(&self) -> Arc<SceneGraph> {
        self.scene_graph.clone()
    }

    /// Get the avatar manager
    pub fn avatar_manager(&self) -> Arc<avatar::AvatarManager> {
        self.avatar_manager.clone()
    }

    /// Get the snapshot manager
    pub fn snapshot_manager(&self) -> Arc<SnapshotManager> {
        self.snapshot_manager.clone()
    }

    /// Get the policy manager
    pub fn policy_manager(&self) -> Arc<PolicyManager> {
        self.policy_manager.clone()
    }

    /// Get the capability manager
    pub fn cap_manager(&self) -> Arc<CapManager> {
        self.cap_manager.clone()
    }

    /// Get the session manager
    pub fn session_manager(&self) -> Arc<SessionManager> {
        self.session_manager.clone()
    }

    /// Get service statistics
    pub async fn get_stats(&self) -> SceneStats {
        self.stats.read().await.clone()
    }

    /// Get service configuration
    pub fn config(&self) -> &SceneConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scene_service_creation() {
        let config = SceneConfig::default();
        let service = SceneService::new(config).await;
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_scene_service_start_stop() {
        let config = SceneConfig::default();
        let service = SceneService::new(config).await.unwrap();
        
        // Start the service
        let start_result = service.start().await;
        assert!(start_result.is_ok());

        // Stop the service
        let stop_result = service.stop().await;
        assert!(stop_result.is_ok());
    }

    #[tokio::test]
    async fn test_scene_service_stats() {
        let config = SceneConfig::default();
        let service = SceneService::new(config).await.unwrap();
        
        let stats = service.get_stats().await;
        assert_eq!(stats.total_nodes, 0);
        assert_eq!(stats.total_avatars, 0);
    }
}
