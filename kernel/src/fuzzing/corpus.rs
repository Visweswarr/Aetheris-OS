/// Fuzzing Corpus Module
/// 
/// Manages the corpus of test inputs for fuzzing.

use alloc::vec::Vec;
use alloc::collections::BTreeSet;

/// Corpus of fuzzing inputs
pub struct Corpus {
    /// Maximum corpus size
    max_size: usize,
    /// Input entries
    inputs: Vec<Vec<u8>>,
    /// Hash set for deduplication
    hashes: BTreeSet<u64>,
}

impl Corpus {
    /// Create a new corpus
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            inputs: Vec::new(),
            hashes: BTreeSet::new(),
        }
    }
    
    /// Add an input to the corpus
    pub fn add_input(&mut self, input: Vec<u8>) {
        let hash = Self::hash_input(&input);
        if !self.hashes.contains(&hash) && self.inputs.len() < self.max_size {
            self.hashes.insert(hash);
            self.inputs.push(input);
        }
    }
    
    /// Add a crash-inducing input
    pub fn add_crash_input(&mut self, input: Vec<u8>) {
        // Crash inputs are always added
        self.inputs.push(input);
    }
    
    /// Select an input from the corpus
    pub fn select_input(&self) -> Vec<u8> {
        if self.inputs.is_empty() {
            Vec::new()
        } else {
            let idx = (crate::rng::random() as usize) % self.inputs.len();
            self.inputs[idx].clone()
        }
    }
    
    /// Get corpus size
    pub fn size(&self) -> usize {
        self.inputs.len()
    }
    
    /// Generate corpus summary
    pub fn generate_summary(&self) -> super::CorpusSummary {
        super::CorpusSummary {
            total_inputs: self.inputs.len(),
            unique_inputs: self.hashes.len(),
            input_types: alloc::collections::BTreeMap::new(),
            size_distribution: Vec::new(),
            coverage_contribution: Vec::new(),
        }
    }
    
    /// Hash an input for deduplication
    fn hash_input(input: &[u8]) -> u64 {
        let mut hash: u64 = 0;
        for &byte in input {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
        }
        hash
    }
}
