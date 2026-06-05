/// Fuzzing Mutation Module
/// 
/// Provides mutation strategies for fuzzing inputs.

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

/// Mutation engine
pub struct MutationEngine {
    /// Mutation rate (0.0 - 1.0)
    mutation_rate: f32,
    /// Total mutations applied
    mutation_count: AtomicU64,
}

impl MutationEngine {
    /// Create a new mutation engine
    pub fn new(mutation_rate: f32) -> Self {
        Self {
            mutation_rate,
            mutation_count: AtomicU64::new(0),
        }
    }
    
    /// Mutate an input
    pub fn mutate(&self, input: &[u8]) -> Vec<u8> {
        self.mutation_count.fetch_add(1, Ordering::Relaxed);
        
        let mut result = input.to_vec();
        if result.is_empty() {
            return result;
        }
        
        // Apply random mutation
        let mutation_type = (crate::rng::random() % 5) as u8;
        
        match mutation_type {
            0 => self.bit_flip(&mut result),
            1 => self.byte_flip(&mut result),
            2 => self.insert_byte(&mut result),
            3 => self.delete_byte(&mut result),
            _ => self.swap_bytes(&mut result),
        }
        
        result
    }
    
    /// Flip a random bit
    fn bit_flip(&self, data: &mut Vec<u8>) {
        if data.is_empty() { return; }
        let idx = (crate::rng::random() as usize) % data.len();
        let bit = (crate::rng::random() % 8) as u8;
        data[idx] ^= 1 << bit;
    }
    
    /// Flip a random byte
    fn byte_flip(&self, data: &mut Vec<u8>) {
        if data.is_empty() { return; }
        let idx = (crate::rng::random() as usize) % data.len();
        data[idx] ^= 0xFF;
    }
    
    /// Insert a random byte
    fn insert_byte(&self, data: &mut Vec<u8>) {
        let idx = (crate::rng::random() as usize) % (data.len() + 1);
        let byte = (crate::rng::random() % 256) as u8;
        data.insert(idx, byte);
    }
    
    /// Delete a random byte
    fn delete_byte(&self, data: &mut Vec<u8>) {
        if data.is_empty() { return; }
        let idx = (crate::rng::random() as usize) % data.len();
        data.remove(idx);
    }
    
    /// Swap two random bytes
    fn swap_bytes(&self, data: &mut Vec<u8>) {
        if data.len() < 2 { return; }
        let idx1 = (crate::rng::random() as usize) % data.len();
        let idx2 = (crate::rng::random() as usize) % data.len();
        data.swap(idx1, idx2);
    }
    
    /// Get mutation count
    pub fn get_mutation_count(&self) -> u64 {
        self.mutation_count.load(Ordering::Relaxed)
    }
}
