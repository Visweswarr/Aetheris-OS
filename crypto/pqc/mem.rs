use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

/// Memory safety configuration
pub struct MemoryConfig {
    /// Enable zeroization on drop
    pub zeroize_on_drop: bool,
    /// Enable memory leak detection
    pub leak_detection: bool,
    /// Enable bounds checking
    pub bounds_checking: bool,
    /// Enable memory isolation
    pub memory_isolation: bool,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            zeroize_on_drop: true,
            leak_detection: cfg!(debug_assertions),
            bounds_checking: true,
            memory_isolation: true,
        }
    }
}

/// Global memory configuration
static MEMORY_CONFIG: Mutex<MemoryConfig> = Mutex::new(MemoryConfig::default());

/// Memory leak detection state
static LEAK_DETECTION_ENABLED: AtomicBool = AtomicBool::new(cfg!(debug_assertions));

/// Get current memory configuration
pub fn get_memory_config() -> MemoryConfig {
    MEMORY_CONFIG.lock().unwrap().clone()
}

/// Set memory configuration
pub fn set_memory_config(config: MemoryConfig) {
    *MEMORY_CONFIG.lock().unwrap() = config;
}

/// Enable/disable leak detection
pub fn set_leak_detection(enabled: bool) {
    LEAK_DETECTION_ENABLED.store(enabled, Ordering::SeqCst);
}

/// Check if leak detection is enabled
pub fn leak_detection_enabled() -> bool {
    LEAK_DETECTION_ENABLED.load(Ordering::SeqCst)
}

/// Secure memory region with automatic zeroization
pub struct SecureMemory<T> {
    data: T,
    zeroized: bool,
}

impl<T> SecureMemory<T> {
    /// Create new secure memory region
    pub fn new(data: T) -> Self {
        Self {
            data,
            zeroized: false,
        }
    }

    /// Get reference to data
    pub fn as_ref(&self) -> &T {
        &self.data
    }

    /// Get mutable reference to data
    pub fn as_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Consume and return data
    pub fn into_inner(mut self) -> T {
        self.zeroize();
        self.data
    }

    /// Check if memory has been zeroized
    pub fn is_zeroized(&self) -> bool {
        self.zeroized
    }
}

impl<T> Drop for SecureMemory<T> {
    fn drop(&mut self) {
        if !self.zeroized {
            self.zeroize();
        }
    }
}

impl<T> SecureMemory<T> {
    /// Zeroize the memory region
    fn zeroize(&mut self) {
        if get_memory_config().zeroize_on_drop {
            unsafe {
                // Use volatile write to prevent optimization
                let ptr = &mut self.data as *mut T as *mut u8;
                let size = std::mem::size_of::<T>();
                
                for i in 0..size {
                    ptr::write_volatile(ptr.add(i), 0u8);
                }
                
                // Compiler barrier to prevent reordering
                std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
            }
            self.zeroized = true;
        }
    }
}

/// Memory leak detector for debugging
#[cfg(debug_assertions)]
pub struct LeakDetector {
    allocations: Mutex<std::collections::HashMap<usize, AllocationInfo>>,
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
struct AllocationInfo {
    size: usize,
    location: String,
    timestamp: std::time::Instant,
}

#[cfg(debug_assertions)]
impl LeakDetector {
    /// Create new leak detector
    pub fn new() -> Self {
        Self {
            allocations: Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// Track memory allocation
    pub fn track_allocation(&self, ptr: usize, size: usize, location: &str) {
        if leak_detection_enabled() {
            let info = AllocationInfo {
                size,
                location: location.to_string(),
                timestamp: std::time::Instant::now(),
            };
            
            self.allocations.lock().unwrap().insert(ptr, info);
        }
    }

    /// Track memory deallocation
    pub fn track_deallocation(&self, ptr: usize) {
        if leak_detection_enabled() {
            self.allocations.lock().unwrap().remove(&ptr);
        }
    }

    /// Get current allocations
    pub fn get_allocations(&self) -> Vec<AllocationInfo> {
        self.allocations.lock().unwrap().values().cloned().collect()
    }

    /// Check for memory leaks
    pub fn check_for_leaks(&self) -> bool {
        let allocations = self.get_allocations();
        if !allocations.is_empty() {
            eprintln!("Memory leaks detected:");
            for info in allocations {
                eprintln!("  - {} bytes at {} (allocated {:?} ago)", 
                    info.size, info.location, info.timestamp.elapsed());
            }
            false
        } else {
            true
        }
    }
}

#[cfg(debug_assertions)]
impl Drop for LeakDetector {
    fn drop(&mut self) {
        if !self.check_for_leaks() {
            panic!("Memory leaks detected during shutdown");
        }
    }
}

#[cfg(not(debug_assertions))]
pub struct LeakDetector;

#[cfg(not(debug_assertions))]
impl LeakDetector {
    pub fn new() -> Self { Self }
    pub fn track_allocation(&self, _ptr: usize, _size: usize, _location: &str) {}
    pub fn track_deallocation(&self, _ptr: usize) {}
    pub fn get_allocations(&self) -> Vec<AllocationInfo> { vec![] }
    pub fn check_for_leaks(&self) -> bool { true }
}

/// Global leak detector instance
static LEAK_DETECTOR: once_cell::sync::Lazy<LeakDetector> = 
    once_cell::sync::Lazy::new(LeakDetector::new);

/// Get global leak detector
pub fn get_leak_detector() -> &'static LeakDetector {
    &LEAK_DETECTOR
}

/// Memory allocation wrapper with leak detection
pub fn secure_alloc<T>(size: usize, location: &str) -> *mut T {
    let ptr = unsafe { std::alloc::alloc_zeroed(std::alloc::Layout::from_size_align(size, 8).unwrap()) };
    
    if !ptr.is_null() {
        get_leak_detector().track_allocation(ptr as usize, size, location);
    }
    
    ptr as *mut T
}

/// Memory deallocation wrapper with leak detection
pub unsafe fn secure_dealloc<T>(ptr: *mut T, size: usize) {
    if !ptr.is_null() {
        get_leak_detector().track_deallocation(ptr as usize);
        
        // Zeroize before deallocation
        if get_memory_config().zeroize_on_drop {
            let ptr_u8 = ptr as *mut u8;
            for i in 0..size {
                ptr::write_volatile(ptr_u8.add(i), 0u8);
            }
            std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
        }
        
        std::alloc::dealloc(ptr as *mut u8, std::alloc::Layout::from_size_align(size, 8).unwrap());
    }
}

/// Memory reallocation wrapper with leak detection
pub unsafe fn secure_realloc<T>(ptr: *mut T, old_size: usize, new_size: usize, location: &str) -> *mut T {
    if !ptr.is_null() {
        get_leak_detector().track_deallocation(ptr as usize);
    }
    
    let new_ptr = std::alloc::realloc(
        ptr as *mut u8,
        std::alloc::Layout::from_size_align(old_size, 8).unwrap(),
        new_size
    ) as *mut T;
    
    if !new_ptr.is_null() {
        get_leak_detector().track_allocation(new_ptr as usize, new_size, location);
    }
    
    new_ptr
}

/// Memory safety utilities
pub mod utils {
    use super::*;

    /// Check if memory region contains only zeros
    pub fn is_zeroized(data: &[u8]) -> bool {
        data.iter().all(|&b| b == 0)
    }

    /// Check if memory region contains any non-zero bytes
    pub fn has_non_zero(data: &[u8]) -> bool {
        data.iter().any(|&b| b != 0)
    }

    /// Securely compare two memory regions (constant-time)
    pub fn secure_compare(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        
        let mut result = 0u8;
        for (x, y) in a.iter().zip(b.iter()) {
            result |= x ^ y;
        }
        
        result == 0
    }

    /// Securely copy memory with zeroization of source
    pub fn secure_copy<T>(src: &mut T, dst: &mut T) {
        unsafe {
            std::ptr::copy_nonoverlapping(src, dst, 1);
            // Zeroize source
            let src_ptr = src as *mut T as *mut u8;
            let size = std::mem::size_of::<T>();
            for i in 0..size {
                ptr::write_volatile(src_ptr.add(i), 0u8);
            }
            std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
        }
    }

    /// Securely move memory with zeroization of source
    pub fn secure_move<T>(src: &mut T, dst: &mut T) {
        secure_copy(src, dst);
        // Source is already zeroized by secure_copy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_memory_zeroization() {
        let config = MemoryConfig {
            zeroize_on_drop: true,
            leak_detection: false,
            bounds_checking: true,
            memory_isolation: true,
        };
        set_memory_config(config);

        let data = vec![1u8, 2, 3, 4, 5];
        let ptr = data.as_ptr() as usize;
        
        {
            let _secure = SecureMemory::new(data);
        } // Should zeroize on drop
        
        // Verify memory is zeroized (this is unsafe and test-only)
        unsafe {
            let memory_region = std::slice::from_raw_parts(ptr as *const u8, 5);
            assert!(utils::is_zeroized(memory_region));
        }
    }

    #[test]
    fn test_leak_detection() {
        set_leak_detection(true);
        let detector = LeakDetector::new();
        
        // Track allocations
        detector.track_allocation(0x1000, 64, "test1");
        detector.track_allocation(0x2000, 128, "test2");
        
        // Check allocations
        let allocations = detector.get_allocations();
        assert_eq!(allocations.len(), 2);
        
        // Track deallocation
        detector.track_deallocation(0x1000);
        
        // Check remaining allocations
        let allocations = detector.get_allocations();
        assert_eq!(allocations.len(), 1);
        
        // Clean up
        detector.track_deallocation(0x2000);
        assert!(detector.check_for_leaks());
    }

    #[test]
    fn test_memory_utilities() {
        let zeros = vec![0u8; 10];
        let non_zeros = vec![1u8, 2, 3, 4, 5];
        
        assert!(utils::is_zeroized(&zeros));
        assert!(!utils::is_zeroized(&non_zeros));
        assert!(!utils::has_non_zero(&zeros));
        assert!(utils::has_non_zero(&non_zeros));
        
        let a = vec![1u8, 2, 3];
        let b = vec![1u8, 2, 3];
        let c = vec![1u8, 2, 4];
        
        assert!(utils::secure_compare(&a, &b));
        assert!(!utils::secure_compare(&a, &c));
    }

    #[test]
    fn test_secure_copy() {
        let mut src = vec![1u8, 2, 3, 4, 5];
        let mut dst = vec![0u8; 5];
        
        utils::secure_copy(&mut src, &mut dst);
        
        // Destination should contain source data
        assert_eq!(dst, vec![1u8, 2, 3, 4, 5]);
        
        // Source should be zeroized
        assert!(utils::is_zeroized(&src));
    }
}
