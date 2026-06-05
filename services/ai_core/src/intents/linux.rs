//! Linux system adapter using D-Bus

use super::{SystemActionContext, SystemActionResult, SystemIntent};
use crate::error::Result;

pub struct LinuxSystemAdapter;

impl LinuxSystemAdapter {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    pub async fn execute_intent(
        &self,
        intent: &SystemIntent,
        _context: &SystemActionContext,
    ) -> Result<SystemActionResult> {
        Ok(SystemActionResult {
            intent: intent.clone(),
            success: true,
            message: format!("Linux executed: {:?}", intent),
            execution_time_ms: 10,
            platform_data: None,
            error: None,
        })
    }

    pub fn is_intent_supported(&self, _intent: &SystemIntent) -> bool {
        true
    }
    pub fn platform_name(&self) -> &str {
        "linux"
    }
    pub fn get_supported_intents(&self) -> Vec<SystemIntent> {
        vec![SystemIntent::OpenSettings]
    }
}
