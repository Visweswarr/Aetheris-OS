//! Memory allocation error handling for Polymera OS kernel

use core::alloc::Layout;
use crate::kprintln;

/// Allocation error handler
#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    kprintln!("ALLOCATION ERROR: Failed to allocate {} bytes (alignment: {})", 
             layout.size(), layout.align());
    
    // Print memory statistics if available
    #[cfg(feature = "mm_stats")]
    {
        let stats = crate::mm::get_memory_stats();
        kprintln!("Memory stats: used_physical={}, available_physical={}, allocated_pages={}", 
                 stats.used_physical, stats.available_physical, stats.allocated_pages);
    }
    
    // Trigger panic to get full system state dump
    panic!("Memory allocation failed for layout: {:?}", layout);
}
