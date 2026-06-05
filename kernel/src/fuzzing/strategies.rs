/// Fuzzing Strategies Module
/// 
/// Provides different fuzzing strategies.

use alloc::vec::Vec;

/// Fuzzing strategy trait
pub trait FuzzingStrategy {
    /// Generate next input
    fn generate_input(&mut self) -> Vec<u8>;
    
    /// Get strategy name
    fn name(&self) -> &'static str;
}

/// Random fuzzing strategy
pub struct RandomStrategy {
    /// Maximum input size
    max_size: usize,
}

impl RandomStrategy {
    /// Create a new random strategy
    pub fn new(max_size: usize) -> Self {
        Self { max_size }
    }
}

impl FuzzingStrategy for RandomStrategy {
    fn generate_input(&mut self) -> Vec<u8> {
        let size = (crate::rng::random() as usize) % self.max_size;
        let mut input = Vec::with_capacity(size);
        for _ in 0..size {
            input.push((crate::rng::random() % 256) as u8);
        }
        input
    }
    
    fn name(&self) -> &'static str {
        "random"
    }
}

/// Boundary value strategy
pub struct BoundaryStrategy;

impl BoundaryStrategy {
    /// Create a new boundary strategy
    pub fn new() -> Self {
        Self
    }
}

impl FuzzingStrategy for BoundaryStrategy {
    fn generate_input(&mut self) -> Vec<u8> {
        // Generate boundary values
        let boundary_values: &[&[u8]] = &[
            &[0x00],
            &[0xFF],
            &[0x00, 0x00, 0x00, 0x00],
            &[0xFF, 0xFF, 0xFF, 0xFF],
            &[0x7F, 0xFF, 0xFF, 0xFF],
            &[0x80, 0x00, 0x00, 0x00],
        ];
        
        let idx = (crate::rng::random() as usize) % boundary_values.len();
        boundary_values[idx].to_vec()
    }
    
    fn name(&self) -> &'static str {
        "boundary"
    }
}
