//! Compositor Service for Polymera OS
//!
//! Implements:
//! - Scene graph (16.2)
//! - Frame ready handling (16.4)
//! - Display hotplug (16.5)

pub mod scene_graph;
pub mod display;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Surface ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SurfaceId(pub u64);

/// Window ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub u64);

/// Display ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DisplayId(pub u64);

/// Compositor state
pub struct Compositor {
    scene: scene_graph::SceneGraph,
    displays: HashMap<DisplayId, display::Display>,
    next_surface_id: u64,
    next_window_id: u64,
}

impl Compositor {
    pub fn new() -> Self {
        Self {
            scene: scene_graph::SceneGraph::new(),
            displays: HashMap::new(),
            next_surface_id: 1,
            next_window_id: 1,
        }
    }
    
    /// Create a new surface
    pub fn create_surface(&mut self, width: u32, height: u32) -> SurfaceId {
        let id = SurfaceId(self.next_surface_id);
        self.next_surface_id += 1;
        
        let surface = scene_graph::Surface {
            id,
            width,
            height,
            buffer: vec![0u8; (width * height * 4) as usize],
            dirty: true,
        };
        
        self.scene.add_surface(surface);
        id
    }
    
    /// Create a window
    pub fn create_window(&mut self, surface_id: SurfaceId, x: i32, y: i32) -> WindowId {
        let id = WindowId(self.next_window_id);
        self.next_window_id += 1;
        
        let window = scene_graph::Window {
            id,
            surface_id,
            x,
            y,
            z_order: self.next_window_id as i32,
            visible: true,
        };
        
        self.scene.add_window(window);
        id
    }
    
    /// Handle frame ready (16.4)
    pub fn frame_ready(&mut self, surface_id: SurfaceId) {
        if let Some(surface) = self.scene.get_surface_mut(surface_id) {
            surface.dirty = true;
        }
        
        // Trigger render
        self.render();
    }
    
    /// Render the scene
    pub fn render(&mut self) {
        // In production: use Vulkan to composite all visible windows
        println!("[COMPOSITOR] Rendering {} windows", self.scene.window_count());
        
        // Mark surfaces as clean
        self.scene.mark_clean();
    }
    
    /// Handle display hotplug (16.5)
    pub fn handle_hotplug(&mut self, display_id: DisplayId, connected: bool) {
        if connected {
            let display = display::Display::new(display_id, 1920, 1080);
            self.displays.insert(display_id, display);
            println!("[COMPOSITOR] Display {} connected", display_id.0);
        } else {
            self.displays.remove(&display_id);
            println!("[COMPOSITOR] Display {} disconnected", display_id.0);
        }
    }
}
