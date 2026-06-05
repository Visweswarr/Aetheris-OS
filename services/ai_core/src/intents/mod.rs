//! System Intents → OS Actions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::cap::CapTokenManager;
use crate::error::{AiCoreError, Result};

pub mod linux;
pub mod mock;
pub mod windows;

/// Wrapper for f32 that implements Eq and Hash (for use in enums)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OrderedFloat(pub f32);

impl PartialEq for OrderedFloat {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}
impl Eq for OrderedFloat {}
impl std::hash::Hash for OrderedFloat {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}
impl From<f32> for OrderedFloat {
    fn from(f: f32) -> Self {
        Self(f)
    }
}
impl From<OrderedFloat> for f32 {
    fn from(f: OrderedFloat) -> Self {
        f.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SystemIntent {
    OpenSettings,
    OpenSettingsCategory(String),
    ToggleBluetooth(bool),
    ToggleWifi(bool),
    ToggleAirplaneMode(bool),
    AdjustVolume(OrderedFloat),
    SetBrightness(OrderedFloat),
    CreateNote,
    OpenFileManager,
    OpenBrowser,
    OpenTerminal,
    LockScreen,
    Shutdown,
    Restart,
    Sleep,
    Custom(String, HashMap<String, serde_json::Value>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemActionResult {
    pub intent: SystemIntent,
    pub success: bool,
    pub message: String,
    pub execution_time_ms: u64,
    pub platform_data: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemActionContext {
    pub user_id: String,
    pub session_id: String,
    pub cap_token: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Concrete adapter enum for object safety (avoids dyn trait with async)
pub enum PlatformAdapter {
    Mock(mock::MockSystemAdapter),
    Linux(linux::LinuxSystemAdapter),
    Windows(windows::WindowsSystemAdapter),
}

impl PlatformAdapter {
    pub async fn execute_intent(
        &self,
        intent: &SystemIntent,
        context: &SystemActionContext,
    ) -> Result<SystemActionResult> {
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
        Self {
            adapter,
            cap_token_manager,
            intent_capabilities: HashMap::new(),
        }
    }

    pub fn parse_intent(&self, intent_text: &str) -> Result<SystemIntent> {
        let text = intent_text.to_lowercase();
        let enabled = !(text.contains("off")
            || text.contains("disable")
            || text.contains("disabled")
            || text.contains("deactivate"));
        if text.contains("settings") {
            for category in ["display", "sound", "network", "bluetooth", "wifi", "privacy"] {
                if text.contains(category) {
                    return Ok(SystemIntent::OpenSettingsCategory(category.to_string()));
                }
            }
            return Ok(SystemIntent::OpenSettings);
        }
        if text.contains("bluetooth") {
            return Ok(SystemIntent::ToggleBluetooth(enabled));
        }
        if text.contains("wifi") || text.contains("wi-fi") {
            return Ok(SystemIntent::ToggleWifi(enabled));
        }
        if text.contains("airplane") || text.contains("flight mode") {
            return Ok(SystemIntent::ToggleAirplaneMode(enabled));
        }
        if text.contains("volume") {
            let value = parse_level(&text).unwrap_or_else(|| {
                if text.contains("down") {
                    -0.1
                } else if text.contains("up") {
                    0.1
                } else {
                    0.5
                }
            });
            return Ok(SystemIntent::AdjustVolume(OrderedFloat(value)));
        }
        if text.contains("brightness") {
            let value = parse_level(&text).unwrap_or_else(|| {
                if text.contains("down") {
                    -0.1
                } else if text.contains("up") {
                    0.1
                } else {
                    0.5
                }
            });
            return Ok(SystemIntent::SetBrightness(OrderedFloat(value)));
        }
        if text.contains("note") {
            return Ok(SystemIntent::CreateNote);
        }
        if text.contains("file") || text.contains("files") || text.contains("explorer") {
            return Ok(SystemIntent::OpenFileManager);
        }
        if text.contains("browser") || text.contains("web") || text.contains("internet") {
            return Ok(SystemIntent::OpenBrowser);
        }
        if text.contains("terminal") || text.contains("cmd") || text.trim() == "open command" {
            return Ok(SystemIntent::OpenTerminal);
        }
        if text.contains("lock") {
            return Ok(SystemIntent::LockScreen);
        }
        if text.contains("shutdown") || text.contains("shut down") {
            return Ok(SystemIntent::Shutdown);
        }
        if text.contains("restart") || text.contains("reboot") {
            return Ok(SystemIntent::Restart);
        }
        if text.contains("sleep") || text.contains("suspend") {
            return Ok(SystemIntent::Sleep);
        }
        let mut params = HashMap::new();
        params.insert(
            "original_text".to_string(),
            serde_json::Value::String(intent_text.to_string()),
        );
        Ok(SystemIntent::Custom("unknown".to_string(), params))
    }

    pub async fn execute_intent(
        &self,
        intent: &SystemIntent,
        context: &SystemActionContext,
    ) -> Result<SystemActionResult> {
        if !self.adapter.is_intent_supported(intent) {
            return Err(AiCoreError::NotSupported(format!(
                "Intent {:?} not supported on {}",
                intent,
                self.adapter.platform_name()
            )));
        }
        if !self.get_required_capabilities(intent).is_empty() && context.cap_token.is_none() {
            return Err(AiCoreError::CapabilityDenied(
                "Capability token required".to_string(),
            ));
        }
        self.adapter.execute_intent(intent, context).await
    }

    pub fn get_supported_intents(&self) -> Vec<SystemIntent> {
        self.adapter.get_supported_intents()
    }
    pub fn platform_name(&self) -> &str {
        self.adapter.platform_name()
    }

    pub fn get_required_capabilities(&self, intent: &SystemIntent) -> Vec<String> {
        match intent {
            SystemIntent::OpenSettings | SystemIntent::OpenSettingsCategory(_) => {
                vec!["settings.read".to_string()]
            }
            SystemIntent::ToggleBluetooth(_) => vec!["hardware.bluetooth.modify".to_string()],
            SystemIntent::ToggleWifi(_) => vec!["hardware.wifi.modify".to_string()],
            SystemIntent::ToggleAirplaneMode(_) => vec!["hardware.radio.modify".to_string()],
            SystemIntent::AdjustVolume(_) => vec!["hardware.audio.modify".to_string()],
            SystemIntent::SetBrightness(_) => vec!["hardware.display.modify".to_string()],
            SystemIntent::CreateNote => vec!["filesystem.write".to_string()],
            SystemIntent::OpenFileManager => vec!["filesystem.read".to_string()],
            SystemIntent::OpenBrowser => vec!["network.access".to_string()],
            SystemIntent::OpenTerminal => vec!["system.shell.execute".to_string()],
            SystemIntent::LockScreen => vec!["system.session.control".to_string()],
            SystemIntent::Shutdown => vec!["system.power.shutdown".to_string()],
            SystemIntent::Restart => vec!["system.power.restart".to_string()],
            SystemIntent::Sleep => vec!["system.power.sleep".to_string()],
            SystemIntent::Custom(_, _) => Vec::new(),
        }
    }
}

fn parse_level(text: &str) -> Option<f32> {
    for token in text.split_whitespace() {
        let token = token.trim_matches(|c: char| c == ',' || c == '.');
        if let Some(percent) = token.strip_suffix('%') {
            if let Ok(value) = percent.parse::<f32>() {
                return Some(value / 100.0);
            }
        }
        if let Ok(value) = token.parse::<f32>() {
            return Some(if value > 1.0 { value / 100.0 } else { value });
        }
    }
    None
}

pub fn create_system_intent_manager(
    cap_token_manager: Arc<CapTokenManager>,
) -> Result<SystemIntentManager> {
    let adapter = Arc::new(PlatformAdapter::Mock(mock::MockSystemAdapter::new()));
    Ok(SystemIntentManager::new(adapter, cap_token_manager))
}
