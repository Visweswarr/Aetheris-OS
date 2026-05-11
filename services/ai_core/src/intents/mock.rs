//! Mock system adapter for testing

use crate::error::Result;
use super::{SystemIntent, SystemActionResult, SystemActionContext, OrderedFloat};

pub struct MockSystemAdapter;

impl Default for MockSystemAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl MockSystemAdapter {
    pub fn new() -> Self { Self }

    pub async fn execute_intent(&self, intent: &SystemIntent, _context: &SystemActionContext) -> Result<SystemActionResult> {
        Ok(SystemActionResult {
            intent: intent.clone(), success: true, message: format!("Mock executed: {:?}", intent),
            execution_time_ms: 10, platform_data: None, error: None,
        })
    }

    pub fn is_intent_supported(&self, _intent: &SystemIntent) -> bool { true }
    pub fn platform_name(&self) -> &str { "mock" }
    
    pub fn get_supported_intents(&self) -> Vec<SystemIntent> {
        vec![
            SystemIntent::OpenSettings, SystemIntent::ToggleBluetooth(true),
            SystemIntent::ToggleWifi(true), SystemIntent::AdjustVolume(OrderedFloat(0.5)),
            SystemIntent::SetBrightness(OrderedFloat(0.5)), SystemIntent::CreateNote,
            SystemIntent::OpenFileManager, SystemIntent::OpenBrowser,
            SystemIntent::OpenTerminal, SystemIntent::LockScreen,
        ]
    }
}
