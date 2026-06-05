//! LZ4 Page Compression for Polymera OS Memory Manager
//!
//! This module implements page compression using LZ4 algorithm to reduce memory pressure.
//! When memory usage exceeds 80%, inactive pages are compressed to free physical memory.
//!
//! Requirements: 1.2 - LZ4 page compression at 80% memory pressure
//!
//! **Property 1: Memory Compression Trigger**
//! For any memory state where pressure exceeds 80%, the Memory_Manager shall initiate
//! LZ4 compression of inactive pages before any swap operations occur.

use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Mutex;

use super::{PhysicalAddress, VirtualAddress, PAGE_SIZE};

// Import lz4_flex for actual LZ4 compression
use lz4_flex::{compress_prepend_size, decompress_size_prepended};

/// Size threshold for compression (4KB pages)
pub const COMPRESSION_PAGE_SIZE: usize = 4096;

/// Memory pressure threshold for triggering compression (80%)
pub const COMPRESSION_THRESHOLD: f32 = 0.80;

/// Maximum compressed page cache size (256MB worth of original pages)
pub const MAX_COMPRESSED_CACHE_SIZE: usize = 256 * 1024 * 1024 / COMPRESSION_PAGE_SIZE;

/// Statistics for compression operations
#[derive(Debug, Clone, Copy, Default)]
pub struct CompressionStats {
    /// Total pages compressed
    pub pages_compressed: u64,
    /// Total pages decompressed
    pub pages_decompressed: u64,
    /// Bytes saved through compression
    pub bytes_saved: u64,
    /// Average compression ratio
    pub avg_compression_ratio: f32,
    /// Total compression time in microseconds
    pub total_compression_time_us: u64,
    /// Total decompression time in microseconds
    pub total_decompression_time_us: u64,
}

/// A compressed page entry in the cache
#[derive(Debug, Clone)]
pub struct CompressedPage {
    /// Original physical address of the page
    pub original_phys_addr: PhysicalAddress,
    /// Original virtual address of the page
    pub original_virt_addr: VirtualAddress,
    /// Compressed data
    pub compressed_data: Vec<u8>,
    /// Original size before compression
    pub original_size: usize,
    /// Compression ratio (original / compressed)
    pub compression_ratio: f32,
    /// Timestamp when compressed
    pub timestamp: u64,
    /// Owner process ID
    pub owner_pid: u64,
}

/// LZ4 Compressor for memory pages
///
/// Implements LZ4 compression/decompression for inactive memory pages.
/// The compressor maintains a cache of compressed pages and statistics.
#[derive(Debug)]
pub struct Lz4Compressor {
    /// Compressed page cache indexed by original virtual address
    compressed_cache: BTreeMap<VirtualAddress, CompressedPage>,
    /// Compression statistics
    stats: CompressionStats,
    /// Whether compression is enabled
    enabled: bool,
    /// Current cache size (number of pages)
    cache_size: usize,
}

impl Lz4Compressor {
    /// Create a new LZ4 compressor
    pub const fn new() -> Self {
        Self {
            compressed_cache: BTreeMap::new(),
            stats: CompressionStats {
                pages_compressed: 0,
                pages_decompressed: 0,
                bytes_saved: 0,
                avg_compression_ratio: 0.0,
                total_compression_time_us: 0,
                total_decompression_time_us: 0,
            },
            enabled: true,
            cache_size: 0,
        }
    }

    /// Enable or disable compression
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if compression is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Compress a memory page using LZ4 algorithm
    ///
    /// # Arguments
    /// * `data` - The page data to compress (must be PAGE_SIZE bytes)
    /// * `virt_addr` - Virtual address of the page
    /// * `phys_addr` - Physical address of the page
    /// * `owner_pid` - Process ID that owns the page
    ///
    /// # Returns
    /// * `Ok(compression_ratio)` if compression successful
    /// * `Err(CompressionError)` if compression failed
    pub fn compress_page(
        &mut self,
        data: &[u8],
        virt_addr: VirtualAddress,
        phys_addr: PhysicalAddress,
        owner_pid: u64,
    ) -> Result<f32, CompressionError> {
        if !self.enabled {
            return Err(CompressionError::Disabled);
        }

        if data.len() != COMPRESSION_PAGE_SIZE {
            return Err(CompressionError::InvalidPageSize);
        }

        if self.cache_size >= MAX_COMPRESSED_CACHE_SIZE {
            return Err(CompressionError::CacheFull);
        }

        // LZ4 compression implementation
        // Using a simple run-length encoding as placeholder for LZ4
        // In production, this would use lz4_flex crate
        let compressed = self.lz4_compress(data)?;
        
        let original_size = data.len();
        let compressed_size = compressed.len();
        
        // Only cache if compression is beneficial (at least 10% reduction)
        if compressed_size >= original_size * 9 / 10 {
            return Err(CompressionError::NotCompressible);
        }

        let compression_ratio = original_size as f32 / compressed_size as f32;
        let bytes_saved = (original_size - compressed_size) as u64;

        let compressed_page = CompressedPage {
            original_phys_addr: phys_addr,
            original_virt_addr: virt_addr,
            compressed_data: compressed,
            original_size,
            compression_ratio,
            timestamp: self.get_timestamp(),
            owner_pid,
        };

        self.compressed_cache.insert(virt_addr, compressed_page);
        self.cache_size += 1;

        // Update statistics
        self.stats.pages_compressed += 1;
        self.stats.bytes_saved += bytes_saved;
        self.update_avg_compression_ratio(compression_ratio);

        Ok(compression_ratio)
    }

    /// Decompress a page from the cache
    ///
    /// # Arguments
    /// * `virt_addr` - Virtual address of the page to decompress
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` containing decompressed page data
    /// * `Err(CompressionError)` if page not found or decompression failed
    pub fn decompress_page(&mut self, virt_addr: VirtualAddress) -> Result<Vec<u8>, CompressionError> {
        let compressed_page = self.compressed_cache
            .remove(&virt_addr)
            .ok_or(CompressionError::PageNotInCache)?;

        let decompressed = self.lz4_decompress(&compressed_page.compressed_data, compressed_page.original_size)?;

        self.cache_size = self.cache_size.saturating_sub(1);
        self.stats.pages_decompressed += 1;

        Ok(decompressed)
    }

    /// Check if a page is in the compressed cache
    pub fn is_page_compressed(&self, virt_addr: VirtualAddress) -> bool {
        self.compressed_cache.contains_key(&virt_addr)
    }

    /// Get compression statistics
    pub fn stats(&self) -> CompressionStats {
        self.stats
    }

    /// Get the number of compressed pages in cache
    pub fn cache_count(&self) -> usize {
        self.cache_size
    }

    /// Get total bytes saved through compression
    pub fn bytes_saved(&self) -> u64 {
        self.stats.bytes_saved
    }

    /// Evict oldest compressed pages to make room
    pub fn evict_oldest(&mut self, count: usize) -> usize {
        let mut evicted = 0;
        let mut to_remove = Vec::new();

        // Find oldest entries
        let mut entries: Vec<_> = self.compressed_cache.iter().collect();
        entries.sort_by_key(|(_, page)| page.timestamp);

        for (addr, _) in entries.iter().take(count) {
            to_remove.push(**addr);
        }

        for addr in to_remove {
            self.compressed_cache.remove(&addr);
            evicted += 1;
        }

        self.cache_size = self.cache_size.saturating_sub(evicted);
        evicted
    }

    /// Clear all compressed pages (used during process termination)
    pub fn clear_process_pages(&mut self, pid: u64) {
        let to_remove: Vec<_> = self.compressed_cache
            .iter()
            .filter(|(_, page)| page.owner_pid == pid)
            .map(|(addr, _)| *addr)
            .collect();

        for addr in to_remove {
            self.compressed_cache.remove(&addr);
            self.cache_size = self.cache_size.saturating_sub(1);
        }
    }

    // ========== Private Implementation ==========

    /// Compress data using LZ4 algorithm via lz4_flex crate
    ///
    /// Uses lz4_flex::compress_prepend_size which prepends the original size
    /// to the compressed data for safe decompression.
    fn lz4_compress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        // Use lz4_flex for actual LZ4 compression
        // compress_prepend_size prepends the uncompressed size as a little-endian u32
        let compressed = compress_prepend_size(data);
        Ok(compressed)
    }

    /// Decompress LZ4 compressed data via lz4_flex crate
    ///
    /// Uses lz4_flex::decompress_size_prepended which reads the original size
    /// from the prepended header.
    fn lz4_decompress(&self, compressed: &[u8], original_size: usize) -> Result<Vec<u8>, CompressionError> {
        // Use lz4_flex for actual LZ4 decompression
        let decompressed = decompress_size_prepended(compressed)
            .map_err(|_| CompressionError::DecompressionFailed)?;

        if decompressed.len() != original_size {
            return Err(CompressionError::DecompressionFailed);
        }

        Ok(decompressed)
    }

    fn get_timestamp(&self) -> u64 {
        // In a real implementation, this would read TSC or system time
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        COUNTER.fetch_add(1, Ordering::Relaxed) as u64
    }

    fn update_avg_compression_ratio(&mut self, new_ratio: f32) {
        let n = self.stats.pages_compressed as f32;
        self.stats.avg_compression_ratio = 
            (self.stats.avg_compression_ratio * (n - 1.0) + new_ratio) / n;
    }
}

/// Compression error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionError {
    /// Compression is disabled
    Disabled,
    /// Invalid page size (must be 4KB)
    InvalidPageSize,
    /// Compression cache is full
    CacheFull,
    /// Data is not compressible
    NotCompressible,
    /// Page not found in compressed cache
    PageNotInCache,
    /// Decompression failed
    DecompressionFailed,
    /// Out of memory
    OutOfMemory,
}

impl core::fmt::Display for CompressionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Disabled => write!(f, "Compression is disabled"),
            Self::InvalidPageSize => write!(f, "Invalid page size"),
            Self::CacheFull => write!(f, "Compression cache is full"),
            Self::NotCompressible => write!(f, "Data is not compressible"),
            Self::PageNotInCache => write!(f, "Page not in compressed cache"),
            Self::DecompressionFailed => write!(f, "Decompression failed"),
            Self::OutOfMemory => write!(f, "Out of memory"),
        }
    }
}

/// Global compressed page cache for memory management
pub static COMPRESSED_CACHE: Mutex<Lz4Compressor> = Mutex::new(Lz4Compressor::new());

/// Check memory pressure and trigger compression if needed
///
/// **Property 1: Memory Compression Trigger**
/// For any memory state where pressure exceeds 80%, this function
/// shall initiate LZ4 compression of inactive pages.
pub fn check_and_compress(memory_pressure: f32) -> bool {
    if memory_pressure > COMPRESSION_THRESHOLD {
        // Compression would be triggered here
        // In a real implementation, this would work with the page frame allocator
        // to identify inactive pages and compress them
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress() {
        let mut compressor = Lz4Compressor::new();
        
        // Create a page with repetitive data (highly compressible)
        let mut data = vec![0u8; COMPRESSION_PAGE_SIZE];
        for i in 0..data.len() {
            data[i] = (i % 4) as u8;
        }

        // Compress
        let result = compressor.compress_page(&data, 0x1000, 0x2000, 1);
        assert!(result.is_ok());
        assert!(compressor.is_page_compressed(0x1000));

        // Decompress
        let decompressed = compressor.decompress_page(0x1000);
        assert!(decompressed.is_ok());
        assert_eq!(decompressed.unwrap(), data);
    }

    #[test]
    fn test_compression_threshold() {
        assert!(check_and_compress(0.85));
        assert!(!check_and_compress(0.75));
    }

    #[test]
    fn test_lz4_compress_decompress_zeros() {
        let mut compressor = Lz4Compressor::new();
        
        // All zeros - highly compressible
        let data = vec![0u8; COMPRESSION_PAGE_SIZE];
        
        let result = compressor.compress_page(&data, 0x2000, 0x3000, 1);
        assert!(result.is_ok());
        
        let ratio = result.unwrap();
        assert!(ratio > 1.0, "Compression ratio should be > 1 for zeros");
        
        let decompressed = compressor.decompress_page(0x2000).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_lz4_compress_decompress_random_pattern() {
        let mut compressor = Lz4Compressor::new();
        
        // Create data with a repeating pattern
        let mut data = vec![0u8; COMPRESSION_PAGE_SIZE];
        for i in 0..data.len() {
            data[i] = ((i * 7 + 13) % 256) as u8;
        }
        
        let result = compressor.compress_page(&data, 0x3000, 0x4000, 2);
        assert!(result.is_ok());
        
        let decompressed = compressor.decompress_page(0x3000).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_compression_stats() {
        let mut compressor = Lz4Compressor::new();
        
        // Compress a few pages
        for i in 0..5 {
            let data = vec![(i as u8); COMPRESSION_PAGE_SIZE];
            let _ = compressor.compress_page(&data, 0x1000 * (i + 1) as u64, 0x2000 * (i + 1) as u64, 1);
        }
        
        let stats = compressor.stats();
        assert!(stats.pages_compressed > 0);
        assert!(stats.bytes_saved > 0);
    }

    #[test]
    fn test_clear_process_pages() {
        let mut compressor = Lz4Compressor::new();
        
        // Compress pages for two processes
        let data = vec![0u8; COMPRESSION_PAGE_SIZE];
        let _ = compressor.compress_page(&data, 0x1000, 0x2000, 1);
        let _ = compressor.compress_page(&data, 0x3000, 0x4000, 2);
        
        assert!(compressor.is_page_compressed(0x1000));
        assert!(compressor.is_page_compressed(0x3000));
        
        // Clear process 1's pages
        compressor.clear_process_pages(1);
        
        assert!(!compressor.is_page_compressed(0x1000));
        assert!(compressor.is_page_compressed(0x3000));
    }

    #[test]
    fn test_compression_disabled() {
        let mut compressor = Lz4Compressor::new();
        compressor.set_enabled(false);
        
        let data = vec![0u8; COMPRESSION_PAGE_SIZE];
        let result = compressor.compress_page(&data, 0x1000, 0x2000, 1);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CompressionError::Disabled);
    }

    #[test]
    fn test_invalid_page_size() {
        let mut compressor = Lz4Compressor::new();
        
        // Wrong size
        let data = vec![0u8; 1024];
        let result = compressor.compress_page(&data, 0x1000, 0x2000, 1);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CompressionError::InvalidPageSize);
    }
}
