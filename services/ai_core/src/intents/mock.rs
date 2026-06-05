//! Mock system adapter for testing

use super::{OrderedFloat, SystemActionContext, SystemActionResult, SystemIntent};
use crate::error::Result;

pub struct MockSystemAdapter;

impl Default for MockSystemAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl MockSystemAdapter {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute_intent(
        &self,
        intent: &SystemIntent,
        _context: &SystemActionContext,
    ) -> Result<SystemActionResult> {
        let message = match intent {
            SystemIntent::ToggleBluetooth(enabled) => {
                format!("Mock: Bluetooth {}", if *enabled { "enabled" } else { "disabled" })
            }
            SystemIntent::ToggleWifi(enabled) => {
                format!("Mock: WiFi {}", if *enabled { "enabled" } else { "disabled" })
            }
            SystemIntent::AdjustVolume(value) => {
                format!("Mock: Volume set to {}%", (value.0 * 100.0).round() as i32)
            }
            SystemIntent::SetBrightness(value) => {
                format!("Mock: Brightness set to {}%", (value.0 * 100.0).round() as i32)
            }
            SystemIntent::Custom(name, _) => format!("Mock: Custom intent '{}'", name),
            _ => format!("Mock: executed {:?}", intent),
        };
        Ok(SystemActionResult {
            intent: intent.clone(),
            success: true,
            message,
            execution_time_ms: 10,
            platform_data: Some(serde_json::json!({
                "platform": "mock",
                "mock": true,
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            })),
            error: None,
        })
    }

    pub fn is_intent_supported(&self, _intent: &SystemIntent) -> bool {
        true
    }
    pub fn platform_name(&self) -> &str {
        "mock"
    }

    pub fn get_supported_intents(&self) -> Vec<SystemIntent> {
        vec![
            SystemIntent::OpenSettings,
            SystemIntent::ToggleBluetooth(true),
            SystemIntent::ToggleBluetooth(false),
            SystemIntent::ToggleWifi(true),
            SystemIntent::ToggleWifi(false),
            SystemIntent::ToggleAirplaneMode(true),
            SystemIntent::ToggleAirplaneMode(false),
            SystemIntent::AdjustVolume(OrderedFloat(0.5)),
            SystemIntent::SetBrightness(OrderedFloat(0.5)),
            SystemIntent::CreateNote,
            SystemIntent::OpenFileManager,
            SystemIntent::OpenBrowser,
            SystemIntent::OpenTerminal,
            SystemIntent::LockScreen,
            SystemIntent::Shutdown,
            SystemIntent::Restart,
            SystemIntent::Sleep,
        ]
    }
}
