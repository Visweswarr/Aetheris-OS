//! Display Management
//!
//! Requirement: 16.5 - Display hotplug

use super::DisplayId;

/// Display mode
#[derive(Debug, Clone, Copy)]
pub struct DisplayMode {
    pub width: u32,
    pub height: u32,
    pub refresh_hz: u32,
}

/// Physical display
#[derive(Debug, Clone)]
pub struct Display {
    pub id: DisplayId,
    pub current_mode: DisplayMode,
    pub available_modes: Vec<DisplayMode>,
    pub connected: bool,
}

impl Display {
    pub fn new(id: DisplayId, width: u32, height: u32) -> Self {
        let mode = DisplayMode {
            width,
            height,
            refresh_hz: 60,
        };
        
        Self {
            id,
            current_mode: mode,
            available_modes: vec![mode],
            connected: true,
        }
    }
    
    /// Set display mode
    pub fn set_mode(&mut self, mode: DisplayMode) -> Result<(), DisplayError> {
        // Validate mode is available
        if !self.available_modes.iter().any(|m| 
            m.width == mode.width && m.height == mode.height) {
            return Err(DisplayError::ModeNotSupported);
        }
        
        self.current_mode = mode;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayError {
    ModeNotSupported,
    DisplayNotFound,
}
