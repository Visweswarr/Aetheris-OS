//! System Intents → OS Actions

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::error::{AiCoreError, Result};
use crate::cap::CapTokenManager;

pub mod linux;
pub mod windows;
pub mod mock;

/// Wrapper for f32 that implements Eq and Hash (for use in enums)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OrderedFloat(pub f32);

impl PartialEq for OrderedFloat {
    fn eq(&self, other: &Self) -> bool { self.0.to_bits() == other.0.to_bits() }
}
impl Eq for OrderedFloat {}
impl std::hash::Hash for OrderedFloat {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.0.to_bits().hash(state); }
}
impl From<f32> for OrderedFloat { fn from(f: f32) -> Self { Self(f) } }
impl From<OrderedFloat> for f32 { fn from(f: OrderedFloat) -> Self { f.0 } }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SystemIntent {
    OpenSettings, OpenSettingsCategory(String), ToggleBluetooth(bool), ToggleWifi(bool),
    ToggleAirplaneMode(bool), AdjustVolume(OrderedFloat), SetBrightness(OrderedFloat), CreateNote,
    OpenFileManager, OpenBrowser, OpenTerminal, LockScreen, Shutdown, Restart, Sleep,
    Custom(String, HashMap<String, serde_json::Value>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemActionResult {
    pub intent: SystemIntent, pub success: bool, pub message: String,
    pub execution_time_ms: u64, pub platform_data: Option<serde_json::Value>, pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SystemActionContext {
    pub user_id: String, pub session_id: String, pub cap_token: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Concrete adapter enum for object safety (avoids dyn trait with async)
pub enum PlatformAdapter {
    Mock(mock::MockSystemAdapter),
    Linux(linux::LinuxSystemAdapter),
    Windows(windows::WindowsSystemAdapter),
}


impl PlatformAdapter {
    pub async fn execute_intent(&self, intent: &SystemIntent, context: &SystemActionContext) -> Result<SystemActionResult> {
        match self {
            PlatformAdapter::Mock(a) => a.execute_intent(intent, context).await,
            PlatformAdapter::Linux(a) => a.execute_intent(intent, context).await,
            PlatformAdapter::Windows(a) => a.execute_intent(intent, context).await,
        }
    }
    pub fn is_intent_supported(&self, intent: &SystemIntent) -> bool {
        match self {
            PlatformAdapter::Mock(a) => a.is_intent_supported(intent),
            PlatformAdapter::Linux(a) => a.is_intent_supported(intent),
            PlatformAdapter::Windows(a) => a.is_intent_supported(intent),
        }
    }
    pub fn platform_name(&self) -> &str {
        match self {
            PlatformAdapter::Mock(a) => a.platform_name(),
            PlatformAdapter::Linux(a) => a.platform_name(),
            PlatformAdapter::Windows(a) => a.platform_name(),
        }
    }
    pub fn get_supported_intents(&self) -> Vec<SystemIntent> {
        match self {
            PlatformAdapter::Mock(a) => a.get_supported_intents(),
            PlatformAdapter::Linux(a) => a.get_supported_intents(),
            PlatformAdapter::Windows(a) => a.get_supported_intents(),
        }
    }
}

pub struct SystemIntentManager {
    adapter: Arc<PlatformAdapter>,
    cap_token_manager: Arc<CapTokenManager>,
    intent_capabilities: HashMap<String, Vec<String>>,
}

impl SystemIntentManager {
    pub fn new(adapter: Arc<PlatformAdapter>, cap_token_manager: Arc<CapTokenManager>) -> Self {
        Self { adapter, cap_token_manager, intent_capabilities: HashMap::new() }
    }

    pub fn parse_intent(&self, intent_text: &str) -> Result<SystemIntent> {
        let text = intent_text.to_lowercase();
        if text.contains("settings") { return Ok(SystemIntent::OpenSettings); }
        if text.contains("bluetooth") { return Ok(SystemIntent::ToggleBluetooth(!text.contains("off"))); }
        if text.contains("wifi") { return Ok(SystemIntent::ToggleWifi(!text.contains("off"))); }
        if text.contains("volume") { return Ok(SystemIntent::AdjustVolume(OrderedFloat(0.5))); }
        if text.contains("brightness") { return Ok(SystemIntent::SetBrightness(OrderedFloat(0.5))); }
        if text.contains("note") { return Ok(SystemIntent::CreateNote); }
        if text.contains("file") { return Ok(SystemIntent::OpenFileManager); }
        if text.contains("browser") { return Ok(SystemIntent::OpenBrowser); }
        if text.contains("terminal") { return Ok(SystemIntent::OpenTerminal); }
        if text.contains("lock") { return Ok(SystemIntent::LockScreen); }
        if text.contains("shutdown") { return Ok(SystemIntent::Shutdown); }
        if text.contains("restart") { return Ok(SystemIntent::Restart); }
        if text.contains("sleep") { return Ok(SystemIntent::Sleep); }
        let mut params = HashMap::new();
        params.insert("original_text".to_string(), serde_json::Value::String(intent_text.to_string()));
        Ok(SystemIntent::Custom("unknown".to_string(), params))
    }

    pub async fn execute_intent(&self, intent: &SystemIntent, context: &SystemActionContext) -> Result<SystemActionResult> {
        if !self.adapter.is_intent_supported(intent) {
            return Err(AiCoreError::NotSupported(format!("Intent {:?} not supported on {}", intent, self.adapter.platform_name())));
        }
        self.adapter.execute_intent(intent, context).await
    }

    pub fn get_supported_intents(&self) -> Vec<SystemIntent> { self.adapter.get_supported_intents() }
    pub fn platform_name(&self) -> &str { self.adapter.platform_name() }
}

pub fn create_system_intent_manager(cap_token_manager: Arc<CapTokenManager>) -> Result<SystemIntentManager> {
    let adapter = Arc::new(PlatformAdapter::Mock(mock::MockSystemAdapter::new()));
    Ok(SystemIntentManager::new(adapter, cap_token_manager))
}
