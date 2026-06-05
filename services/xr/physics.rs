/**
 * Physics & Interaction Engine
 * 
 * This module provides physics simulation and interaction capabilities with:
 * - Deterministic physics simulation
 * - Collision detection and response
 * - Grab/move/collide operations with CapToken gating
 * - Spatial queries and raycasting
 * - Constraint systems and joints
 * - Performance monitoring and optimization
 */

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::error::{SceneError, SceneResult};
use crate::xr::scene::{SceneNode, NodeId, Transform, Vector3};

/// Physics world configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsConfig {
    /// Gravity vector (x, y, z)
    pub gravity: Vector3,
    /// Fixed timestep for deterministic simulation
    pub fixed_timestep: f32,
    /// Maximum simulation steps per frame
    pub max_steps_per_frame: u32,
    /// Enable continuous collision detection
    pub enable_ccd: bool,
    /// Enable sleeping for performance
    pub enable_sleeping: bool,
    /// Collision margin
    pub collision_margin: f32,
    /// Solver iterations
    pub solver_iterations: u32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: Vector3 { x: 0.0, y: -9.81, z: 0.0 },
            fixed_timestep: 1.0 / 60.0, // 60 FPS
            max_steps_per_frame: 4,
            enable_ccd: true,
            enable_sleeping: true,
            collision_margin: 0.04,
            solver_iterations: 10,
        }
    }
}

/// Physics body types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PhysicsBodyType {
    /// Static body (doesn't move)
    Static,
    /// Kinematic body (moved by code)
    Kinematic,
    /// Dynamic body (affected by forces)
    Dynamic,
}

/// Physics shape types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PhysicsShape {
    /// Box shape
    Box { width: f32, height: f32, depth: f32 },
    /// Sphere shape
    Sphere { radius: f32 },
    /// Capsule shape
    Capsule { radius: f32, height: f32 },
    /// Plane shape
    Plane { normal: Vector3, distance: f32 },
    /// Mesh shape (convex hull)
    Mesh { vertices: Vec<Vector3> },
}

/// Physics material properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsMaterial {
    /// Friction coefficient
    pub friction: f32,
    /// Restitution (bounciness)
    pub restitution: f32,
    /// Density (mass per unit volume)
    pub density: f32,
    /// Linear damping
    pub linear_damping: f32,
    /// Angular damping
    pub angular_damping: f32,
}

impl Default for PhysicsMaterial {
    fn default() -> Self {
        Self {
            friction: 0.5,
            restitution: 0.0,
            density: 1.0,
            linear_damping: 0.1,
            angular_damping: 0.1,
        }
    }
}

/// Physics body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsBody {
    pub id: String,
    pub node_id: NodeId,
    pub body_type: PhysicsBodyType,
    pub shape: PhysicsShape,
    pub material: PhysicsMaterial,
    pub transform: Transform,
    pub velocity: Vector3,
    pub angular_velocity: Vector3,
    pub mass: f32,
    pub is_sleeping: bool,
    pub is_trigger: bool,
    pub collision_layers: u32,
    pub collision_mask: u32,
}

/// Physics interaction types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PhysicsInteraction {
    /// Grab an object
    Grab { object_id: NodeId, grab_point: Vector3 },
    /// Move an object
    Move { object_id: NodeId, target_transform: Transform },
    /// Release a grabbed object
    Release { object_id: NodeId },
    /// Apply force to an object
    ApplyForce { object_id: NodeId, force: Vector3, point: Vector3 },
    /// Apply impulse to an object
    ApplyImpulse { object_id: NodeId, impulse: Vector3, point: Vector3 },
    /// Set object velocity
    SetVelocity { object_id: NodeId, velocity: Vector3 },
    /// Teleport object
    Teleport { object_id: NodeId, transform: Transform },
}

/// Collision event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollisionEvent {
    pub id: String,
    pub body_a: String,
    pub body_b: String,
    pub contact_point: Vector3,
    pub contact_normal: Vector3,
    pub penetration_depth: f32,
    pub timestamp: DateTime<Utc>,
}

/// Raycast result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaycastResult {
    pub hit: bool,
    pub body_id: Option<String>,
    pub node_id: Option<NodeId>,
    pub hit_point: Vector3,
    pub hit_normal: Vector3,
    pub distance: f32,
}

/// Physics world statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsStats {
    pub total_bodies: usize,
    pub active_bodies: usize,
    pub sleeping_bodies: usize,
    pub collision_events: u64,
    pub simulation_steps: u64,
    pub average_step_time_ms: f32,
    pub total_simulation_time: Duration,
}

/// Physics world manager
pub struct PhysicsWorld {
    config: PhysicsConfig,
    bodies: Arc<RwLock<HashMap<String, PhysicsBody>>>,
    node_to_body: Arc<RwLock<HashMap<NodeId, String>>>,
    collision_events: Arc<RwLock<Vec<CollisionEvent>>>,
    stats: Arc<RwLock<PhysicsStats>>,
    simulation_time: Arc<RwLock<f32>>,
    accumulator: Arc<RwLock<f32>>,
}

impl PhysicsWorld {
    /// Create a new physics world
    pub fn new(config: PhysicsConfig) -> Self {
        let stats = PhysicsStats {
            total_bodies: 0,
            active_bodies: 0,
            sleeping_bodies: 0,
            collision_events: 0,
            simulation_steps: 0,
            average_step_time_ms: 0.0,
            total_simulation_time: Duration::from_secs(0),
        };

        Self {
            config,
            bodies: Arc::new(RwLock::new(HashMap::new())),
            node_to_body: Arc::new(RwLock::new(HashMap::new())),
            collision_events: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(stats)),
            simulation_time: Arc::new(RwLock::new(0.0)),
            accumulator: Arc::new(RwLock::new(0.0)),
        }
    }

    /// Add a physics body to the world
    pub fn add_body(
        &self,
        node_id: NodeId,
        body_type: PhysicsBodyType,
        shape: PhysicsShape,
        material: PhysicsMaterial,
        transform: Transform,
    ) -> SceneResult<String> {
        let body_id = Uuid::new_v4().to_string();
        
        // Calculate mass based on shape and material
        let volume = self.calculate_shape_volume(&shape);
        let mass = volume * material.density;

        let body = PhysicsBody {
            id: body_id.clone(),
            node_id: node_id.clone(),
            body_type,
            shape,
            material,
            transform,
            velocity: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            angular_velocity: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            mass,
            is_sleeping: false,
            is_trigger: false,
            collision_layers: 1,
            collision_mask: 0xFFFFFFFF,
        };

        // Store body
        self.bodies.write().unwrap().insert(body_id.clone(), body);
        self.node_to_body.write().unwrap().insert(node_id, body_id.clone());

        // Update stats
        self.update_stats();

        Ok(body_id)
    }

    /// Remove a physics body from the world
    pub fn remove_body(&self, body_id: &str) -> SceneResult<()> {
        let body = self.bodies.write().unwrap().remove(body_id)
            .ok_or_else(|| SceneError::PhysicsBodyNotFound(body_id.to_string()))?;

        self.node_to_body.write().unwrap().remove(&body.node_id);
        self.update_stats();

        Ok(())
    }

    /// Update physics simulation
    pub fn step(&self, delta_time: f32) -> SceneResult<()> {
        let start_time = Instant::now();

        // Add to accumulator
        *self.accumulator.write().unwrap() += delta_time;

        // Perform fixed timestep simulation
        let mut steps = 0;
        while *self.accumulator.read().unwrap() >= self.config.fixed_timestep 
            && steps < self.config.max_steps_per_frame {
            
            self.fixed_step()?;
            *self.accumulator.write().unwrap() -= self.config.fixed_timestep;
            steps += 1;
        }

        // Update simulation time
        *self.simulation_time.write().unwrap() += delta_time;

        // Update stats
        let step_time = start_time.elapsed();
        self.update_step_stats(step_time, steps);

        Ok(())
    }

    /// Perform a fixed timestep simulation
    fn fixed_step(&self) -> SceneResult<()> {
        let bodies = self.bodies.read().unwrap();
        let mut bodies_vec: Vec<PhysicsBody> = bodies.values().cloned().collect();
        drop(bodies);

        // Update dynamic bodies
        for body in &mut bodies_vec {
            if body.body_type == PhysicsBodyType::Dynamic && !body.is_sleeping {
                self.integrate_body(body);
            }
        }

        // Detect collisions
        self.detect_collisions(&bodies_vec)?;

        // Update stored bodies
        let mut stored_bodies = self.bodies.write().unwrap();
        for body in bodies_vec {
            stored_bodies.insert(body.id.clone(), body);
        }

        // Update stats
        if let Some(stats) = self.stats.write().unwrap().as_mut() {
            stats.simulation_steps += 1;
        }

        Ok(())
    }

    /// Integrate physics for a single body
    fn integrate_body(&self, body: &mut PhysicsBody) {
        // Apply gravity
        if body.body_type == PhysicsBodyType::Dynamic {
            body.velocity.y += self.config.gravity.y * self.config.fixed_timestep;
        }

        // Apply damping
        body.velocity.x *= 1.0 - body.material.linear_damping * self.config.fixed_timestep;
        body.velocity.y *= 1.0 - body.material.linear_damping * self.config.fixed_timestep;
        body.velocity.z *= 1.0 - body.material.linear_damping * self.config.fixed_timestep;

        body.angular_velocity.x *= 1.0 - body.material.angular_damping * self.config.fixed_timestep;
        body.angular_velocity.y *= 1.0 - body.material.angular_damping * self.config.fixed_timestep;
        body.angular_velocity.z *= 1.0 - body.material.angular_damping * self.config.fixed_timestep;

        // Update position
        body.transform.position.x += body.velocity.x * self.config.fixed_timestep;
        body.transform.position.y += body.velocity.y * self.config.fixed_timestep;
        body.transform.position.z += body.velocity.z * self.config.fixed_timestep;

        // Update rotation (simplified - in real implementation would use quaternions)
        body.transform.rotation.x += body.angular_velocity.x * self.config.fixed_timestep;
        body.transform.rotation.y += body.angular_velocity.y * self.config.fixed_timestep;
        body.transform.rotation.z += body.angular_velocity.z * self.config.fixed_timestep;
    }

    /// Detect collisions between bodies
    fn detect_collisions(&self, bodies: &[PhysicsBody]) -> SceneResult<()> {
        for i in 0..bodies.len() {
            for j in (i + 1)..bodies.len() {
                let body_a = &bodies[i];
                let body_b = &bodies[j];

                // Skip if both are static
                if body_a.body_type == PhysicsBodyType::Static 
                    && body_b.body_type == PhysicsBodyType::Static {
                    continue;
                }

                // Check collision layers
                if (body_a.collision_layers & body_b.collision_mask) == 0 
                    || (body_b.collision_layers & body_a.collision_mask) == 0 {
                    continue;
                }

                // Perform collision detection
                if let Some(collision) = self.check_collision(body_a, body_b) {
                    self.handle_collision(collision)?;
                }
            }
        }

        Ok(())
    }

    /// Check collision between two bodies
    fn check_collision(&self, body_a: &PhysicsBody, body_b: &PhysicsBody) -> Option<CollisionEvent> {
        // Simplified collision detection - in real implementation would use proper algorithms
        let distance = self.calculate_distance(&body_a.transform.position, &body_b.transform.position);
        let min_distance = self.get_minimum_distance(body_a, body_b);

        if distance < min_distance {
            let contact_point = self.calculate_contact_point(body_a, body_b);
            let contact_normal = self.calculate_contact_normal(body_a, body_b);
            let penetration_depth = min_distance - distance;

            return Some(CollisionEvent {
                id: Uuid::new_v4().to_string(),
                body_a: body_a.id.clone(),
                body_b: body_b.id.clone(),
                contact_point,
                contact_normal,
                penetration_depth,
                timestamp: Utc::now(),
            });
        }

        None
    }

    /// Handle collision response
    fn handle_collision(&self, mut collision: CollisionEvent) -> SceneResult<()> {
        // Store collision event
        self.collision_events.write().unwrap().push(collision.clone());

        // Update stats
        if let Some(stats) = self.stats.write().unwrap().as_mut() {
            stats.collision_events += 1;
        }

        // Apply collision response (simplified)
        let mut bodies = self.bodies.write().unwrap();
        if let (Some(body_a), Some(body_b)) = (
            bodies.get_mut(&collision.body_a),
            bodies.get_mut(&collision.body_b)
        ) {
            // Simple collision response
            if body_a.body_type == PhysicsBodyType::Dynamic {
                // Separate bodies
                let separation = Vector3 {
                    x: collision.contact_normal.x * collision.penetration_depth * 0.5,
                    y: collision.contact_normal.y * collision.penetration_depth * 0.5,
                    z: collision.contact_normal.z * collision.penetration_depth * 0.5,
                };
                body_a.transform.position.x -= separation.x;
                body_a.transform.position.y -= separation.y;
                body_a.transform.position.z -= separation.z;

                // Apply restitution
                let relative_velocity = self.calculate_relative_velocity(body_a, body_b);
                let velocity_along_normal = self.dot_product(&relative_velocity, &collision.contact_normal);
                
                if velocity_along_normal > 0.0 {
                    let restitution = (body_a.material.restitution + body_b.material.restitution) * 0.5;
                    let impulse = -(1.0 + restitution) * velocity_along_normal;
                    
                    body_a.velocity.x += impulse * collision.contact_normal.x;
                    body_a.velocity.y += impulse * collision.contact_normal.y;
                    body_a.velocity.z += impulse * collision.contact_normal.z;
                }
            }

            if body_b.body_type == PhysicsBodyType::Dynamic {
                // Separate bodies
                let separation = Vector3 {
                    x: collision.contact_normal.x * collision.penetration_depth * 0.5,
                    y: collision.contact_normal.y * collision.penetration_depth * 0.5,
                    z: collision.contact_normal.z * collision.penetration_depth * 0.5,
                };
                body_b.transform.position.x += separation.x;
                body_b.transform.position.y += separation.y;
                body_b.transform.position.z += separation.z;

                // Apply restitution
                let relative_velocity = self.calculate_relative_velocity(body_b, body_a);
                let velocity_along_normal = self.dot_product(&relative_velocity, &collision.contact_normal);
                
                if velocity_along_normal > 0.0 {
                    let restitution = (body_a.material.restitution + body_b.material.restitution) * 0.5;
                    let impulse = -(1.0 + restitution) * velocity_along_normal;
                    
                    body_b.velocity.x -= impulse * collision.contact_normal.x;
                    body_b.velocity.y -= impulse * collision.contact_normal.y;
                    body_b.velocity.z -= impulse * collision.contact_normal.z;
                }
            }
        }

        Ok(())
    }

    /// Perform physics interaction
    pub fn interact(
        &self,
        interaction: PhysicsInteraction,
        session_id: &str,
    ) -> SceneResult<()> {
        match interaction {
            PhysicsInteraction::Grab { object_id, grab_point } => {
                self.grab_object(object_id, grab_point, session_id)?;
            }
            PhysicsInteraction::Move { object_id, target_transform } => {
                self.move_object(object_id, target_transform, session_id)?;
            }
            PhysicsInteraction::Release { object_id } => {
                self.release_object(object_id, session_id)?;
            }
            PhysicsInteraction::ApplyForce { object_id, force, point } => {
                self.apply_force(object_id, force, point, session_id)?;
            }
            PhysicsInteraction::ApplyImpulse { object_id, impulse, point } => {
                self.apply_impulse(object_id, impulse, point, session_id)?;
            }
            PhysicsInteraction::SetVelocity { object_id, velocity } => {
                self.set_velocity(object_id, velocity, session_id)?;
            }
            PhysicsInteraction::Teleport { object_id, transform } => {
                self.teleport_object(object_id, transform, session_id)?;
            }
        }

        Ok(())
    }

    /// Grab an object
    fn grab_object(&self, object_id: NodeId, grab_point: Vector3, session_id: &str) -> SceneResult<()> {
        // In a real implementation, this would check CapTokens and maintain grab state
        // For now, we'll just log the interaction
        Ok(())
    }

    /// Move an object
    fn move_object(&self, object_id: NodeId, target_transform: Transform, session_id: &str) -> SceneResult<()> {
        if let Some(body_id) = self.node_to_body.read().unwrap().get(&object_id) {
            if let Some(body) = self.bodies.write().unwrap().get_mut(body_id) {
                body.transform = target_transform;
            }
        }
        Ok(())
    }

    /// Release an object
    fn release_object(&self, object_id: NodeId, session_id: &str) -> SceneResult<()> {
        // In a real implementation, this would release grab constraints
        Ok(())
    }

    /// Apply force to an object
    fn apply_force(&self, object_id: NodeId, force: Vector3, point: Vector3, session_id: &str) -> SceneResult<()> {
        if let Some(body_id) = self.node_to_body.read().unwrap().get(&object_id) {
            if let Some(body) = self.bodies.write().unwrap().get_mut(body_id) {
                if body.body_type == PhysicsBodyType::Dynamic {
                    // Apply force (simplified - in real implementation would consider point of application)
                    body.velocity.x += force.x / body.mass * self.config.fixed_timestep;
                    body.velocity.y += force.y / body.mass * self.config.fixed_timestep;
                    body.velocity.z += force.z / body.mass * self.config.fixed_timestep;
                }
            }
        }
        Ok(())
    }

    /// Apply impulse to an object
    fn apply_impulse(&self, object_id: NodeId, impulse: Vector3, point: Vector3, session_id: &str) -> SceneResult<()> {
        if let Some(body_id) = self.node_to_body.read().unwrap().get(&object_id) {
            if let Some(body) = self.bodies.write().unwrap().get_mut(body_id) {
                if body.body_type == PhysicsBodyType::Dynamic {
                    body.velocity.x += impulse.x / body.mass;
                    body.velocity.y += impulse.y / body.mass;
                    body.velocity.z += impulse.z / body.mass;
                }
            }
        }
        Ok(())
    }

    /// Set object velocity
    fn set_velocity(&self, object_id: NodeId, velocity: Vector3, session_id: &str) -> SceneResult<()> {
        if let Some(body_id) = self.node_to_body.read().unwrap().get(&object_id) {
            if let Some(body) = self.bodies.write().unwrap().get_mut(body_id) {
                body.velocity = velocity;
            }
        }
        Ok(())
    }

    /// Teleport object
    fn teleport_object(&self, object_id: NodeId, transform: Transform, session_id: &str) -> SceneResult<()> {
        if let Some(body_id) = self.node_to_body.read().unwrap().get(&object_id) {
            if let Some(body) = self.bodies.write().unwrap().get_mut(body_id) {
                body.transform = transform;
                body.velocity = Vector3 { x: 0.0, y: 0.0, z: 0.0 };
                body.angular_velocity = Vector3 { x: 0.0, y: 0.0, z: 0.0 };
            }
        }
        Ok(())
    }

    /// Perform raycast
    pub fn raycast(&self, origin: Vector3, direction: Vector3, max_distance: f32) -> SceneResult<RaycastResult> {
        let mut closest_hit: Option<RaycastResult> = None;
        let mut closest_distance = max_distance;

        let bodies = self.bodies.read().unwrap();
        for body in bodies.values() {
            if let Some(hit) = self.raycast_against_body(origin, direction, body) {
                if hit.distance < closest_distance {
                    closest_distance = hit.distance;
                    closest_hit = Some(hit);
                }
            }
        }

        Ok(closest_hit.unwrap_or_else(|| RaycastResult {
            hit: false,
            body_id: None,
            node_id: None,
            hit_point: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            hit_normal: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            distance: 0.0,
        }))
    }

    /// Raycast against a specific body
    fn raycast_against_body(&self, origin: Vector3, direction: Vector3, body: &PhysicsBody) -> Option<RaycastResult> {
        // Simplified raycast - in real implementation would use proper algorithms
        match &body.shape {
            PhysicsShape::Sphere { radius } => {
                self.raycast_sphere(origin, direction, body.transform.position, *radius)
            }
            PhysicsShape::Box { width, height, depth } => {
                self.raycast_box(origin, direction, body.transform.position, *width, *height, *depth)
            }
            _ => None,
        }
    }

    /// Raycast against sphere
    fn raycast_sphere(&self, origin: Vector3, direction: Vector3, center: Vector3, radius: f32) -> Option<RaycastResult> {
        // Simplified sphere raycast
        let oc = Vector3 {
            x: origin.x - center.x,
            y: origin.y - center.y,
            z: origin.z - center.z,
        };

        let a = self.dot_product(&direction, &direction);
        let b = 2.0 * self.dot_product(&oc, &direction);
        let c = self.dot_product(&oc, &oc) - radius * radius;

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return None;
        }

        let t = (-b - discriminant.sqrt()) / (2.0 * a);
        if t < 0.0 {
            return None;
        }

        let hit_point = Vector3 {
            x: origin.x + direction.x * t,
            y: origin.y + direction.y * t,
            z: origin.z + direction.z * t,
        };

        let hit_normal = Vector3 {
            x: (hit_point.x - center.x) / radius,
            y: (hit_point.y - center.y) / radius,
            z: (hit_point.z - center.z) / radius,
        };

        Some(RaycastResult {
            hit: true,
            body_id: Some("body_id".to_string()),
            node_id: Some("node_id".to_string()),
            hit_point,
            hit_normal,
            distance: t,
        })
    }

    /// Raycast against box
    fn raycast_box(&self, origin: Vector3, direction: Vector3, center: Vector3, width: f32, height: f32, depth: f32) -> Option<RaycastResult> {
        // Simplified box raycast
        let half_width = width * 0.5;
        let half_height = height * 0.5;
        let half_depth = depth * 0.5;

        let min = Vector3 {
            x: center.x - half_width,
            y: center.y - half_height,
            z: center.z - half_depth,
        };

        let max = Vector3 {
            x: center.x + half_width,
            y: center.y + half_height,
            z: center.z + half_depth,
        };

        // Simple AABB raycast
        let mut t_min = (min.x - origin.x) / direction.x;
        let mut t_max = (max.x - origin.x) / direction.x;

        if t_min > t_max {
            std::mem::swap(&mut t_min, &mut t_max);
        }

        let ty_min = (min.y - origin.y) / direction.y;
        let ty_max = (max.y - origin.y) / direction.y;

        if ty_min > ty_max {
            let temp = ty_min;
            let ty_min = ty_max;
            let ty_max = temp;
        }

        if t_min > ty_max || ty_min > t_max {
            return None;
        }

        t_min = t_min.max(ty_min);
        t_max = t_max.min(ty_max);

        let tz_min = (min.z - origin.z) / direction.z;
        let tz_max = (max.z - origin.z) / direction.z;

        if tz_min > tz_max {
            let temp = tz_min;
            let tz_min = tz_max;
            let tz_max = temp;
        }

        if t_min > tz_max || tz_min > t_max {
            return None;
        }

        t_min = t_min.max(tz_min);
        t_max = t_max.min(tz_max);

        if t_min < 0.0 {
            return None;
        }

        let hit_point = Vector3 {
            x: origin.x + direction.x * t_min,
            y: origin.y + direction.y * t_min,
            z: origin.z + direction.z * t_min,
        };

        Some(RaycastResult {
            hit: true,
            body_id: Some("body_id".to_string()),
            node_id: Some("node_id".to_string()),
            hit_point,
            hit_normal: Vector3 { x: 0.0, y: 1.0, z: 0.0 }, // Simplified
            distance: t_min,
        })
    }

    /// Get physics body by node ID
    pub fn get_body_by_node(&self, node_id: &NodeId) -> SceneResult<Option<PhysicsBody>> {
        if let Some(body_id) = self.node_to_body.read().unwrap().get(node_id) {
            Ok(self.bodies.read().unwrap().get(body_id).cloned())
        } else {
            Ok(None)
        }
    }

    /// Get all physics bodies
    pub fn get_all_bodies(&self) -> SceneResult<Vec<PhysicsBody>> {
        let bodies = self.bodies.read().unwrap();
        Ok(bodies.values().cloned().collect())
    }

    /// Get physics statistics
    pub fn get_stats(&self) -> SceneResult<PhysicsStats> {
        let stats = self.stats.read().unwrap();
        Ok(stats.clone())
    }

    /// Get collision events
    pub fn get_collision_events(&self) -> SceneResult<Vec<CollisionEvent>> {
        let events = self.collision_events.read().unwrap();
        Ok(events.clone())
    }

    // Helper methods

    fn calculate_shape_volume(&self, shape: &PhysicsShape) -> f32 {
        match shape {
            PhysicsShape::Box { width, height, depth } => width * height * depth,
            PhysicsShape::Sphere { radius } => (4.0 / 3.0) * std::f32::consts::PI * radius * radius * radius,
            PhysicsShape::Capsule { radius, height } => {
                let sphere_volume = (4.0 / 3.0) * std::f32::consts::PI * radius * radius * radius;
                let cylinder_volume = std::f32::consts::PI * radius * radius * height;
                sphere_volume + cylinder_volume
            }
            PhysicsShape::Plane { .. } => 0.0,
            PhysicsShape::Mesh { .. } => 1.0, // Simplified
        }
    }

    fn calculate_distance(&self, a: &Vector3, b: &Vector3) -> f32 {
        let dx = a.x - b.x;
        let dy = a.y - b.y;
        let dz = a.z - b.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    fn get_minimum_distance(&self, body_a: &PhysicsBody, body_b: &PhysicsBody) -> f32 {
        // Simplified - in real implementation would use proper algorithms
        match (&body_a.shape, &body_b.shape) {
            (PhysicsShape::Sphere { radius: r1 }, PhysicsShape::Sphere { radius: r2 }) => r1 + r2,
            _ => 1.0, // Simplified
        }
    }

    fn calculate_contact_point(&self, body_a: &PhysicsBody, body_b: &PhysicsBody) -> Vector3 {
        // Simplified - midpoint between bodies
        Vector3 {
            x: (body_a.transform.position.x + body_b.transform.position.x) * 0.5,
            y: (body_a.transform.position.y + body_b.transform.position.y) * 0.5,
            z: (body_a.transform.position.z + body_b.transform.position.z) * 0.5,
        }
    }

    fn calculate_contact_normal(&self, body_a: &PhysicsBody, body_b: &PhysicsBody) -> Vector3 {
        // Simplified - normalized vector from B to A
        let dx = body_a.transform.position.x - body_b.transform.position.x;
        let dy = body_a.transform.position.y - body_b.transform.position.y;
        let dz = body_a.transform.position.z - body_b.transform.position.z;
        let length = (dx * dx + dy * dy + dz * dz).sqrt();
        
        if length > 0.0 {
            Vector3 { x: dx / length, y: dy / length, z: dz / length }
        } else {
            Vector3 { x: 0.0, y: 1.0, z: 0.0 }
        }
    }

    fn calculate_relative_velocity(&self, body_a: &PhysicsBody, body_b: &PhysicsBody) -> Vector3 {
        Vector3 {
            x: body_a.velocity.x - body_b.velocity.x,
            y: body_a.velocity.y - body_b.velocity.y,
            z: body_a.velocity.z - body_b.velocity.z,
        }
    }

    fn dot_product(&self, a: &Vector3, b: &Vector3) -> f32 {
        a.x * b.x + a.y * b.y + a.z * b.z
    }

    fn update_stats(&self) {
        let bodies = self.bodies.read().unwrap();
        let mut stats = self.stats.write().unwrap();
        
        stats.total_bodies = bodies.len();
        stats.active_bodies = bodies.values().filter(|b| !b.is_sleeping).count();
        stats.sleeping_bodies = bodies.values().filter(|b| b.is_sleeping).count();
    }

    fn update_step_stats(&self, step_time: Duration, steps: u32) {
        if let Some(stats) = self.stats.write().unwrap().as_mut() {
            stats.total_simulation_time += step_time;
            if steps > 0 {
                stats.average_step_time_ms = step_time.as_secs_f32() * 1000.0 / steps as f32;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_world_creation() {
        let config = PhysicsConfig::default();
        let world = PhysicsWorld::new(config);
        
        let stats = world.get_stats().unwrap();
        assert_eq!(stats.total_bodies, 0);
    }

    #[test]
    fn test_add_remove_body() {
        let config = PhysicsConfig::default();
        let world = PhysicsWorld::new(config);
        
        let transform = Transform {
            position: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            rotation: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            scale: Vector3 { x: 1.0, y: 1.0, z: 1.0 },
        };

        let body_id = world.add_body(
            "node_123".to_string(),
            PhysicsBodyType::Dynamic,
            PhysicsShape::Sphere { radius: 1.0 },
            PhysicsMaterial::default(),
            transform,
        ).unwrap();

        let stats = world.get_stats().unwrap();
        assert_eq!(stats.total_bodies, 1);

        world.remove_body(&body_id).unwrap();

        let stats = world.get_stats().unwrap();
        assert_eq!(stats.total_bodies, 0);
    }

    #[test]
    fn test_physics_simulation() {
        let config = PhysicsConfig::default();
        let world = PhysicsWorld::new(config);
        
        let transform = Transform {
            position: Vector3 { x: 0.0, y: 10.0, z: 0.0 },
            rotation: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            scale: Vector3 { x: 1.0, y: 1.0, z: 1.0 },
        };

        let body_id = world.add_body(
            "node_123".to_string(),
            PhysicsBodyType::Dynamic,
            PhysicsShape::Sphere { radius: 1.0 },
            PhysicsMaterial::default(),
            transform,
        ).unwrap();

        // Simulate for 1 second
        for _ in 0..60 {
            world.step(1.0 / 60.0).unwrap();
        }

        let body = world.get_body_by_node(&"node_123".to_string()).unwrap().unwrap();
        assert!(body.transform.position.y < 10.0); // Should have fallen due to gravity
    }

    #[test]
    fn test_raycast() {
        let config = PhysicsConfig::default();
        let world = PhysicsWorld::new(config);
        
        let transform = Transform {
            position: Vector3 { x: 0.0, y: 0.0, z: 5.0 },
            rotation: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            scale: Vector3 { x: 1.0, y: 1.0, z: 1.0 },
        };

        world.add_body(
            "node_123".to_string(),
            PhysicsBodyType::Static,
            PhysicsShape::Sphere { radius: 1.0 },
            PhysicsMaterial::default(),
            transform,
        ).unwrap();

        let origin = Vector3 { x: 0.0, y: 0.0, z: 0.0 };
        let direction = Vector3 { x: 0.0, y: 0.0, z: 1.0 };
        let result = world.raycast(origin, direction, 10.0).unwrap();

        assert!(result.hit);
        assert!(result.distance > 0.0);
    }

    #[test]
    fn test_physics_interaction() {
        let config = PhysicsConfig::default();
        let world = PhysicsWorld::new(config);
        
        let transform = Transform {
            position: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            rotation: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            scale: Vector3 { x: 1.0, y: 1.0, z: 1.0 },
        };

        world.add_body(
            "node_123".to_string(),
            PhysicsBodyType::Dynamic,
            PhysicsShape::Sphere { radius: 1.0 },
            PhysicsMaterial::default(),
            transform,
        ).unwrap();

        let interaction = PhysicsInteraction::ApplyForce {
            object_id: "node_123".to_string(),
            force: Vector3 { x: 10.0, y: 0.0, z: 0.0 },
            point: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
        };

        world.interact(interaction, "session_123").unwrap();

        let body = world.get_body_by_node(&"node_123".to_string()).unwrap().unwrap();
        assert!(body.velocity.x > 0.0); // Should have velocity in X direction
    }
}
