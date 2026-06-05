//! Device policy manager for capability validation and DAO policy enforcement

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::DeviceError;

/// Device policy manager that handles capability validation and DAO policy enforcement
pub struct DevicePolicyManager {
    active_sessions: Arc<RwLock<HashMap<String, SessionInfo>>>,
    dao_policies: Arc<RwLock<HashMap<String, DaoPolicy>>>,
}

#[derive(Debug, Clone)]
struct SessionInfo {
    session_id: String,
    did: String,
    capabilities: Vec<String>,
    room_id: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
struct DaoPolicy {
    room_id: String,
    allow_camera: bool,
    allow_microphone: bool,
    allow_preview: bool,
    // BLE policies
    allow_ble_scan: bool,
    allow_ble_connect: bool,
    allow_ble_gatt_read: bool,
    allow_ble_gatt_write: bool,
    allow_ble_notify_subscribe: bool,
    // Sensor policies
    allow_sensor_read: bool,
    allow_sensor_sample: bool,
    allow_sensor_preview: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl DevicePolicyManager {
    pub fn new() -> Result<Self, DeviceError> {
        Ok(Self {
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            dao_policies: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Validate a capability for a session
    pub async fn validate_capability(
        &self,
        session_id: &str,
        caps: &str,
        required_capability: &str,
    ) -> Result<(), DeviceError> {
        // Parse capabilities from caps string
        let session_capabilities = self.parse_capabilities(caps);
        
        // Check if required capability is present
        if !session_capabilities.contains(&required_capability.to_string()) {
            return Err(DeviceError::InvalidCapability(format!(
                "Missing required capability: {}",
                required_capability
            )));
        }
        
        // Store session info for future DAO policy checks
        let session_info = SessionInfo {
            session_id: session_id.to_string(),
            did: "mock_did".to_string(), // TODO: Get from identity service
            capabilities: session_capabilities,
            room_id: None, // TODO: Get from session context
            created_at: chrono::Utc::now(),
        };
        
        self.active_sessions.write().await.insert(session_id.to_string(), session_info);
        
        Ok(())
    }
    
    /// Check DAO policy for device access in shared rooms
    pub async fn check_dao_policy(
        &self,
        session_id: &str,
        operation: &str,
    ) -> Result<(), DeviceError> {
        let session_info = {
            let sessions = self.active_sessions.read().await;
            sessions.get(session_id).cloned()
        };
        
        // If no room context, allow operation (local device access)
        let room_id = match session_info {
            Some(info) => info.room_id,
            None => return Ok(()),
        };
        
        let room_id = match room_id {
            Some(id) => id,
            None => return Ok(()),
        };
        
        // Check DAO policy for the room
        let policy = {
            let policies = self.dao_policies.read().await;
            policies.get(&room_id).cloned()
        };
        
        match policy {
            Some(policy) => {
                let allowed = match operation {
                    "camera_capture" => policy.allow_camera,
                    "microphone_capture" => policy.allow_microphone,
                    "preview" => policy.allow_preview,
                    // BLE operations
                    "ble_scan" => policy.allow_ble_scan,
                    "ble_connect" => policy.allow_ble_connect,
                    "ble_gatt_read" => policy.allow_ble_gatt_read,
                    "ble_gatt_write" => policy.allow_ble_gatt_write,
                    "ble_notify_subscribe" => policy.allow_ble_notify_subscribe,
                    // Sensor operations
                    "sensor_read" => policy.allow_sensor_read,
                    "sensor_sample" => policy.allow_sensor_sample,
                    "sensor_preview" => policy.allow_sensor_preview,
                    _ => false,
                };
                
                if !allowed {
                    return Err(DeviceError::DaoPolicyDenied(format!(
                        "DAO policy denies {} in room {}",
                        operation, room_id
                    )));
                }
            }
            None => {
                // No policy found, create default restrictive policy
                let default_policy = DaoPolicy {
                    room_id: room_id.clone(),
                    allow_camera: false,
                    allow_microphone: false,
                    allow_preview: false,
                    allow_ble_scan: false,
                    allow_ble_connect: false,
                    allow_ble_gatt_read: false,
                    allow_ble_gatt_write: false,
                    allow_ble_notify_subscribe: false,
                    allow_sensor_read: false,
                    allow_sensor_sample: false,
                    allow_sensor_preview: false,
                    created_at: chrono::Utc::now(),
                };
                
                self.dao_policies.write().await.insert(room_id, default_policy);
                
                return Err(DeviceError::DaoPolicyDenied(format!(
                    "No DAO policy found for room {}, defaulting to deny",
                    room_id
                )));
            }
        }
        
        Ok(())
    }
    
    /// Set DAO policy for a room
    pub async fn set_dao_policy(
        &self,
        room_id: &str,
        allow_camera: bool,
        allow_microphone: bool,
        allow_preview: bool,
        allow_ble_scan: bool,
        allow_ble_connect: bool,
        allow_ble_gatt_read: bool,
        allow_ble_gatt_write: bool,
        allow_ble_notify_subscribe: bool,
        allow_sensor_read: bool,
        allow_sensor_sample: bool,
        allow_sensor_preview: bool,
    ) -> Result<(), DeviceError> {
        let policy = DaoPolicy {
            room_id: room_id.to_string(),
            allow_camera,
            allow_microphone,
            allow_preview,
            allow_ble_scan,
            allow_ble_connect,
            allow_ble_gatt_read,
            allow_ble_gatt_write,
            allow_ble_notify_subscribe,
            allow_sensor_read,
            allow_sensor_sample,
            allow_sensor_preview,
            created_at: chrono::Utc::now(),
        };
        
        self.dao_policies.write().await.insert(room_id.to_string(), policy);
        
        tracing::info!(
            "Set DAO policy for room {}: camera={}, mic={}, preview={}, ble_scan={}, ble_connect={}, sensor_read={}, sensor_sample={}",
            room_id, allow_camera, allow_microphone, allow_preview, 
            allow_ble_scan, allow_ble_connect, allow_sensor_read, allow_sensor_sample
        );
        
        Ok(())
    }
    
    /// Get current DAO policy for a room
    pub async fn get_dao_policy(&self, room_id: &str) -> Result<Option<DaoPolicy>, DeviceError> {
        let policies = self.dao_policies.read().await;
        Ok(policies.get(room_id).cloned())
    }
    
    fn parse_capabilities(&self, caps: &str) -> Vec<String> {
        caps.split(',')
            .map(|cap| cap.trim().to_string())
            .filter(|cap| !cap.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_policy_manager_creation() {
        let manager = DevicePolicyManager::new();
        assert!(manager.is_ok());
    }
    
    #[tokio::test]
    async fn test_capability_validation() {
        let manager = DevicePolicyManager::new().unwrap();
        let session_id = "test_session";
        let caps = "device:camera.read,device:mic.read";
        
        // Valid capability
        let result = manager.validate_capability(session_id, caps, "device:camera.read").await;
        assert!(result.is_ok());
        
        // Invalid capability
        let result = manager.validate_capability(session_id, caps, "device:invalid.read").await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_dao_policy() {
        let manager = DevicePolicyManager::new().unwrap();
        let room_id = "test_room";
        
        // Set policy
        manager.set_dao_policy(
            room_id, 
            true, false, true,  // camera, mic, preview
            true, true, true, true, true,  // BLE policies
            true, true, true  // sensor policies
        ).await.unwrap();
        
        // Get policy
        let policy = manager.get_dao_policy(room_id).await.unwrap();
        assert!(policy.is_some());
        
        let policy = policy.unwrap();
        assert!(policy.allow_camera);
        assert!(!policy.allow_microphone);
        assert!(policy.allow_preview);
        assert!(policy.allow_ble_scan);
        assert!(policy.allow_ble_connect);
        assert!(policy.allow_sensor_read);
        assert!(policy.allow_sensor_sample);
    }
}
