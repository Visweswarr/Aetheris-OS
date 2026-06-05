//! Synchronization primitives for the kernel
//! Re-exports spin crate types for convenience

// Re-export spin types with explicit names to avoid conflicts
pub type Mutex<T> = spin::Mutex<T>;
pub type MutexGuard<'a, T> = spin::MutexGuard<'a, T>;
pub type RwLock<T> = spin::RwLock<T>;
pub type RwLockReadGuard<'a, T> = spin::RwLockReadGuard<'a, T>;
pub type RwLockWriteGuard<'a, T> = spin::RwLockWriteGuard<'a, T>;
