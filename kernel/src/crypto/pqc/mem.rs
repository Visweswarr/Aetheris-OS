//! Secure memory handling stubs

use alloc::vec::Vec;

/// Secure memory wrapper that zeros on drop
pub struct SecureMemory {
    data: Vec<u8>,
}

impl SecureMemory {
    pub fn new(size: usize) -> Self {
        Self {
            data: alloc::vec![0u8; size],
        }
    }
    
    pub fn from_slice(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }
    
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
    
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Drop for SecureMemory {
    fn drop(&mut self) {
        // Zero out memory on drop
        for byte in &mut self.data {
            *byte = 0;
        }
    }
}
