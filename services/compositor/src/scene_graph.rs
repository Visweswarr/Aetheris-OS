//! Scene Graph Implementation
//!
//! Requirement: 16.2 - Scene graph (Rust)

use super::{SurfaceId, WindowId};
use std::collections::HashMap;

/// Surface - a buffer that can be drawn to
#[derive(Debug, Clone)]
pub struct Surface {
    pub id: SurfaceId,
    pub width: u32,
    pub height: u32,
    pub buffer: Vec<u8>,
    pub dirty: bool,
}

/// Window - a positioned surface in the scene
#[derive(Debug, Clone)]
pub struct Window {
    pub id: WindowId,
    pub surface_id: SurfaceId,
    pub x: i32,
    pub y: i32,
    pub z_order: i32,
    pub visible: bool,
}

/// Scene Graph - manages all surfaces and windows
pub struct SceneGraph {
    surfaces: HashMap<SurfaceId, Surface>,
    windows: HashMap<WindowId, Window>,
    /// Sorted window order for rendering
    render_order: Vec<WindowId>,
}

impl SceneGraph {
    pub fn new() -> Self {
        Self {
            surfaces: HashMap::new(),
            windows: HashMap::new(),
            render_order: Vec::new(),
        }
    }
    
    /// Add a surface
    pub fn add_surface(&mut self, surface: Surface) {
        self.surfaces.insert(surface.id, surface);
    }
    
    /// Add a window
    pub fn add_window(&mut self, window: Window) {
        let id = window.id;
        self.windows.insert(id, window);
        self.render_order.push(id);
        self.sort_render_order();
    }
    
    /// Get mutable surface
    pub fn get_surface_mut(&mut self, id: SurfaceId) -> Option<&mut Surface> {
        self.surfaces.get_mut(&id)
    }
    
    /// Remove a window
    pub fn remove_window(&mut self, id: WindowId) {
        self.windows.remove(&id);
        self.render_order.retain(|&w| w != id);
    }
    
    /// Sort windows by z-order
    fn sort_render_order(&mut self) {
        self.render_order.sort_by(|a, b| {
            let wa = self.windows.get(a).map(|w| w.z_order).unwrap_or(0);
            let wb = self.windows.get(b).map(|w| w.z_order).unwrap_or(0);
            wa.cmp(&wb)
        });
    }
    
    /// Get windows in render order
    pub fn get_render_order(&self) -> Vec<&Window> {
        self.render_order
            .iter()
            .filter_map(|id| self.windows.get(id))
            .filter(|w| w.visible)
            .collect()
    }
    
    /// Mark all surfaces as clean
    pub fn mark_clean(&mut self) {
        for surface in self.surfaces.values_mut() {
            surface.dirty = false;
        }
    }
    
    /// Count windows
    pub fn window_count(&self) -> usize {
        self.windows.len()
    }
}
