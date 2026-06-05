//! DID Session Management for XR Scene Service
//! 
//! This module handles the lifecycle of authenticated DID sessions for avatars
//! in the XR scene service. It provides session creation, validation, refresh,
//! and revocation capabilities with proper security and audit logging.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{SceneError, SceneResult};
use crate::avatar::{AvatarId, AvatarProfile};

/// Unique identifier for a DID session
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub String);

impl SessionId {
    /// Generate a new unique session ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

/// DID proof for session authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidProof {
    /// The DID being authenticated
    pub did: String,
    /// JWT or other proof format
    pub proof: String,
    /// Nonce for replay protection
    pub nonce: String,
    /// Timestamp when proof was created
    pub timestamp: u64,
}

/// Session state and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Unique session identifier
    pub id: SessionId,
    /// Associated DID
    pub did: String,
    /// Avatar ID bound to this session
    pub avatar_id: Option<AvatarId>,
    /// Session creation time
    pub created_at: u64,
    /// Last activity timestamp
    pub last_activity: u64,
    /// Session expiration time
    pub expires_at: u64,
    /// Whether session is active
    pub is_active: bool,
    /// Session metadata
    pub metadata: HashMap<String, String>,
}

/// Session creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeginSessionRequest {
    /// DID proof for authentication
    pub proof: DidProof,
    /// Optional avatar profile to bind
    pub avatar_profile: Option<AvatarProfile>,
    /// Session TTL in seconds (default: 3600)
    pub ttl_seconds: Option<u64>,
}

/// Session creation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeginSessionResponse {
    /// Created session ID
    pub session_id: SessionId,
    /// Bound avatar ID (if any)
    pub avatar_id: Option<AvatarId>,
    /// Session expiration timestamp
    pub expires_at: u64,
}

/// Session termination request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndSessionRequest {
    /// Session ID to terminate
    pub session_id: SessionId,
    /// Reason for termination
    pub reason: Option<String>,
}

/// Session termination response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndSessionResponse {
    /// Whether termination was successful
    pub success: bool,
    /// Termination timestamp
    pub terminated_at: u64,
}

/// Session refresh request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshSessionRequest {
    /// Session ID to refresh
    pub session_id: SessionId,
    /// New TTL in seconds
    pub ttl_seconds: Option<u64>,
}

/// Session refresh response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshSessionResponse {
    /// Whether refresh was successful
    pub success: bool,
    /// New expiration timestamp
    pub expires_at: u64,
}

/// Session validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionValidation {
    /// Whether session is valid
    pub is_valid: bool,
    /// Session state (if valid)
    pub session: Option<SessionState>,
    /// Validation error (if invalid)
    pub error: Option<String>,
}

/// Session manager for handling DID sessions
pub struct SessionManager {
    /// Active sessions by ID
    sessions: Arc<RwLock<HashMap<SessionId, SessionState>>>,
    /// Sessions by DID for quick lookup
    did_sessions: Arc<RwLock<HashMap<String, Vec<SessionId>>>>,
    /// Default session TTL
    default_ttl: Duration,
    /// Maximum sessions per DID
    max_sessions_per_did: usize,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(default_ttl: Duration, max_sessions_per_did: usize) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            did_sessions: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
            max_sessions_per_did,
        }
    }

    /// Begin a new DID session
    pub fn begin_session(&self, request: BeginSessionRequest) -> SceneResult<BeginSessionResponse> {
        // Validate DID proof
        self.validate_did_proof(&request.proof)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| SceneError::InternalError("System time error".to_string()))?
            .as_secs();

        let ttl = request.ttl_seconds.unwrap_or(self.default_ttl.as_secs());
        let expires_at = now + ttl;

        // Check session limits per DID
        self.check_session_limits(&request.proof.did)?;

        let session_id = SessionId::new();
        let session_state = SessionState {
            id: session_id.clone(),
            did: request.proof.did.clone(),
            avatar_id: None, // Will be set after avatar binding
            created_at: now,
            last_activity: now,
            expires_at,
            is_active: true,
            metadata: HashMap::new(),
        };

        // Store session
        {
            let mut sessions = self.sessions.write().map_err(|_| {
                SceneError::InternalError("Failed to acquire sessions lock".to_string())
            })?;
            sessions.insert(session_id.clone(), session_state);
        }

        // Update DID sessions index
        {
            let mut did_sessions = self.did_sessions.write().map_err(|_| {
                SceneError::InternalError("Failed to acquire did_sessions lock".to_string())
            })?;
            did_sessions
                .entry(request.proof.did)
                .or_insert_with(Vec::new)
                .push(session_id.clone());
        }

        // Emit session started event
        self.emit_session_event("avatar_session_started", &session_id, &request.proof.did)?;

        Ok(BeginSessionResponse {
            session_id,
            avatar_id: None,
            expires_at,
        })
    }

    /// End a DID session
    pub fn end_session(&self, request: EndSessionRequest) -> SceneResult<EndSessionResponse> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| SceneError::InternalError("System time error".to_string()))?
            .as_secs();

        let mut sessions = self.sessions.write().map_err(|_| {
            SceneError::InternalError("Failed to acquire sessions lock".to_string())
        })?;

        if let Some(mut session) = sessions.remove(&request.session_id) {
            session.is_active = false;

            // Remove from DID sessions index
            {
                let mut did_sessions = self.did_sessions.write().map_err(|_| {
                    SceneError::InternalError("Failed to acquire did_sessions lock".to_string())
                })?;
                if let Some(sessions_for_did) = did_sessions.get_mut(&session.did) {
                    sessions_for_did.retain(|id| id != &request.session_id);
                    if sessions_for_did.is_empty() {
                        did_sessions.remove(&session.did);
                    }
                }
            }

            // Emit session ended event
            self.emit_session_event("avatar_session_ended", &request.session_id, &session.did)?;

            Ok(EndSessionResponse {
                success: true,
                terminated_at: now,
            })
        } else {
            Err(SceneError::NotFound(format!(
                "Session {} not found",
                request.session_id.0
            )))
        }
    }

    /// Refresh a session's expiration
    pub fn refresh_session(&self, request: RefreshSessionRequest) -> SceneResult<RefreshSessionResponse> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| SceneError::InternalError("System time error".to_string()))?
            .as_secs();

        let ttl = request.ttl_seconds.unwrap_or(self.default_ttl.as_secs());
        let expires_at = now + ttl;

        let mut sessions = self.sessions.write().map_err(|_| {
            SceneError::InternalError("Failed to acquire sessions lock".to_string())
        })?;

        if let Some(session) = sessions.get_mut(&request.session_id) {
            if !session.is_active {
                return Err(SceneError::InvalidState("Session is not active".to_string()));
            }

            session.last_activity = now;
            session.expires_at = expires_at;

            Ok(RefreshSessionResponse {
                success: true,
                expires_at,
            })
        } else {
            Err(SceneError::NotFound(format!(
                "Session {} not found",
                request.session_id.0
            )))
        }
    }

    /// Validate a session
    pub fn validate_session(&self, session_id: &SessionId) -> SceneResult<SessionValidation> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| SceneError::InternalError("System time error".to_string()))?
            .as_secs();

        let sessions = self.sessions.read().map_err(|_| {
            SceneError::InternalError("Failed to acquire sessions lock".to_string())
        })?;

        if let Some(session) = sessions.get(session_id) {
            if !session.is_active {
                return Ok(SessionValidation {
                    is_valid: false,
                    session: None,
                    error: Some("Session is not active".to_string()),
                });
            }

            if session.expires_at <= now {
                return Ok(SessionValidation {
                    is_valid: false,
                    session: None,
                    error: Some("Session has expired".to_string()),
                });
            }

            Ok(SessionValidation {
                is_valid: true,
                session: Some(session.clone()),
                error: None,
            })
        } else {
            Ok(SessionValidation {
                is_valid: false,
                session: None,
                error: Some("Session not found".to_string()),
            })
        }
    }

    /// Bind an avatar to a session
    pub fn bind_avatar(&self, session_id: &SessionId, avatar_id: AvatarId) -> SceneResult<()> {
        let mut sessions = self.sessions.write().map_err(|_| {
            SceneError::InternalError("Failed to acquire sessions lock".to_string())
        })?;

        if let Some(session) = sessions.get_mut(session_id) {
            if !session.is_active {
                return Err(SceneError::InvalidState("Session is not active".to_string()));
            }

            session.avatar_id = Some(avatar_id);
            session.last_activity = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| SceneError::InternalError("System time error".to_string()))?
                .as_secs();

            Ok(())
        } else {
            Err(SceneError::NotFound(format!(
                "Session {} not found",
                session_id.0
            )))
        }
    }

    /// Get session by ID
    pub fn get_session(&self, session_id: &SessionId) -> SceneResult<SessionState> {
        let sessions = self.sessions.read().map_err(|_| {
            SceneError::InternalError("Failed to acquire sessions lock".to_string())
        })?;

        sessions
            .get(session_id)
            .cloned()
            .ok_or_else(|| SceneError::NotFound(format!("Session {} not found", session_id.0)))
    }

    /// Get all sessions for a DID
    pub fn get_sessions_for_did(&self, did: &str) -> SceneResult<Vec<SessionState>> {
        let sessions = self.sessions.read().map_err(|_| {
            SceneError::InternalError("Failed to acquire sessions lock".to_string())
        })?;

        let did_sessions = self.did_sessions.read().map_err(|_| {
            SceneError::InternalError("Failed to acquire did_sessions lock".to_string())
        })?;

        if let Some(session_ids) = did_sessions.get(did) {
            let mut result = Vec::new();
            for session_id in session_ids {
                if let Some(session) = sessions.get(session_id) {
                    result.push(session.clone());
                }
            }
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&self) -> SceneResult<usize> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| SceneError::InternalError("System time error".to_string()))?
            .as_secs();

        let mut sessions = self.sessions.write().map_err(|_| {
            SceneError::InternalError("Failed to acquire sessions lock".to_string())
        })?;

        let mut did_sessions = self.did_sessions.write().map_err(|_| {
            SceneError::InternalError("Failed to acquire did_sessions lock".to_string())
        })?;

        let mut expired_sessions = Vec::new();

        // Find expired sessions
        for (session_id, session) in sessions.iter() {
            if session.expires_at <= now {
                expired_sessions.push((session_id.clone(), session.did.clone()));
            }
        }

        // Remove expired sessions
        for (session_id, did) in expired_sessions {
            sessions.remove(&session_id);
            
            if let Some(sessions_for_did) = did_sessions.get_mut(&did) {
                sessions_for_did.retain(|id| id != &session_id);
                if sessions_for_did.is_empty() {
                    did_sessions.remove(&did);
                }
            }

            // Emit session expired event
            let _ = self.emit_session_event("avatar_session_expired", &session_id, &did);
        }

        Ok(expired_sessions.len())
    }

    /// Validate DID proof (placeholder implementation)
    fn validate_did_proof(&self, proof: &DidProof) -> SceneResult<()> {
        // TODO: Implement proper DID proof validation
        // This should verify the JWT signature, check nonce, etc.
        
        if proof.did.is_empty() {
            return Err(SceneError::InvalidInput("DID cannot be empty".to_string()));
        }

        if proof.proof.is_empty() {
            return Err(SceneError::InvalidInput("Proof cannot be empty".to_string()));
        }

        if proof.nonce.is_empty() {
            return Err(SceneError::InvalidInput("Nonce cannot be empty".to_string()));
        }

        // Check proof timestamp (should be recent)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| SceneError::InternalError("System time error".to_string()))?
            .as_secs();

        if now.saturating_sub(proof.timestamp) > 300 { // 5 minutes
            return Err(SceneError::InvalidInput("Proof timestamp too old".to_string()));
        }

        Ok(())
    }

    /// Check session limits per DID
    fn check_session_limits(&self, did: &str) -> SceneResult<()> {
        let did_sessions = self.did_sessions.read().map_err(|_| {
            SceneError::InternalError("Failed to acquire did_sessions lock".to_string())
        })?;

        if let Some(sessions) = did_sessions.get(did) {
            if sessions.len() >= self.max_sessions_per_did {
                return Err(SceneError::ResourceLimit(format!(
                    "Maximum sessions per DID exceeded: {}",
                    self.max_sessions_per_did
                )));
            }
        }

        Ok(())
    }

    /// Emit session event (placeholder implementation)
    fn emit_session_event(&self, event_type: &str, session_id: &SessionId, did: &str) -> SceneResult<()> {
        // TODO: Implement proper event emission
        // This should emit events to the event stream
        log::info!(
            "Session event: {} for session {} (DID: {})",
            event_type,
            session_id.0,
            did
        );
        Ok(())
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new(Duration::from_secs(3600), 5) // 1 hour TTL, max 5 sessions per DID
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_session_lifecycle() {
        let manager = SessionManager::new(Duration::from_secs(60), 3);
        
        let proof = DidProof {
            did: "did:aeth:test123".to_string(),
            proof: "jwt_proof_here".to_string(),
            nonce: "nonce123".to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        let request = BeginSessionRequest {
            proof,
            avatar_profile: None,
            ttl_seconds: Some(60),
        };

        // Begin session
        let response = manager.begin_session(request).unwrap();
        assert!(!response.session_id.0.is_empty());
        assert!(response.avatar_id.is_none());

        // Validate session
        let validation = manager.validate_session(&response.session_id).unwrap();
        assert!(validation.is_valid);
        assert!(validation.session.is_some());

        // End session
        let end_request = EndSessionRequest {
            session_id: response.session_id.clone(),
            reason: Some("Test completion".to_string()),
        };

        let end_response = manager.end_session(end_request).unwrap();
        assert!(end_response.success);

        // Validate session is no longer valid
        let validation = manager.validate_session(&response.session_id).unwrap();
        assert!(!validation.is_valid);
    }

    #[test]
    fn test_session_limits() {
        let manager = SessionManager::new(Duration::from_secs(60), 2);
        
        let did = "did:aeth:test456";
        
        // Create first session
        let proof1 = DidProof {
            did: did.to_string(),
            proof: "jwt_proof_1".to_string(),
            nonce: "nonce1".to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        let request1 = BeginSessionRequest {
            proof: proof1,
            avatar_profile: None,
            ttl_seconds: Some(60),
        };

        let response1 = manager.begin_session(request1).unwrap();
        assert!(!response1.session_id.0.is_empty());

        // Create second session
        let proof2 = DidProof {
            did: did.to_string(),
            proof: "jwt_proof_2".to_string(),
            nonce: "nonce2".to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        let request2 = BeginSessionRequest {
            proof: proof2,
            avatar_profile: None,
            ttl_seconds: Some(60),
        };

        let response2 = manager.begin_session(request2).unwrap();
        assert!(!response2.session_id.0.is_empty());

        // Try to create third session (should fail)
        let proof3 = DidProof {
            did: did.to_string(),
            proof: "jwt_proof_3".to_string(),
            nonce: "nonce3".to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        let request3 = BeginSessionRequest {
            proof: proof3,
            avatar_profile: None,
            ttl_seconds: Some(60),
        };

        let result = manager.begin_session(request3);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SceneError::ResourceLimit(_)));
    }

    #[test]
    fn test_session_expiration() {
        let manager = SessionManager::new(Duration::from_millis(100), 3);
        
        let proof = DidProof {
            did: "did:aeth:test789".to_string(),
            proof: "jwt_proof_here".to_string(),
            nonce: "nonce123".to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        let request = BeginSessionRequest {
            proof,
            avatar_profile: None,
            ttl_seconds: Some(1), // 1 second TTL
        };

        let response = manager.begin_session(request).unwrap();
        
        // Session should be valid initially
        let validation = manager.validate_session(&response.session_id).unwrap();
        assert!(validation.is_valid);

        // Wait for expiration
        thread::sleep(Duration::from_millis(1100));

        // Session should be expired
        let validation = manager.validate_session(&response.session_id).unwrap();
        assert!(!validation.is_valid);
        assert!(validation.error.is_some());
        assert!(validation.error.unwrap().contains("expired"));
    }
}
