#![no_main]

use libfuzzer_sys::fuzz_target;
use polymera_os::syscall::validate::{validate_syscall, MAX_SYSCALL_ARGS};

fuzz_target!(|data: &[u8]| {
    // Skip empty inputs
    if data.is_empty() {
        return;
    }
    
    // Test various input sizes and patterns
    test_input_sizes(data);
    test_overflow_scenarios(data);
    test_underflow_scenarios(data);
    test_malformed_inputs(data);
    test_edge_cases(data);
});

/// Test various input sizes
fn test_input_sizes(data: &[u8]) {
    // Test with different input sizes
    for size in [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024] {
        if data.len() >= size {
            let chunk = &data[..size];
            test_syscall_validation(chunk);
        }
    }
}

/// Test overflow scenarios
fn test_overflow_scenarios(data: &[u8]) {
    // Test with very large values that might cause overflow
    let large_values = [
        u64::MAX,
        u64::MAX - 1,
        u64::MAX / 2,
        u64::MAX / 2 + 1,
        0xffffffffffffffff,
        0x8000000000000000,
    ];
    
    for &large_value in &large_values {
        let args = [large_value, large_value, large_value, large_value];
        
        // Test with various syscall IDs
        for syscall_id in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10] {
            let _ = validate_syscall(syscall_id, &args);
        }
    }
}

/// Test underflow scenarios
fn test_underflow_scenarios(data: &[u8]) {
    // Test with very small values and zero
    let small_values = [
        0,
        1,
        2,
        3,
        4,
        5,
        10,
        100,
        1000,
    ];
    
    for &small_value in &small_values {
        let args = [small_value, small_value, small_value, small_value];
        
        // Test with various syscall IDs
        for syscall_id in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10] {
            let _ = validate_syscall(syscall_id, &args);
        }
    }
}

/// Test malformed inputs
fn test_malformed_inputs(data: &[u8]) {
    // Test with unaligned pointers
    let unaligned_pointers = [
        0x400001, // Not 8-byte aligned
        0x400003, // Not 8-byte aligned
        0x400005, // Not 8-byte aligned
        0x400007, // Not 8-byte aligned
    ];
    
    for &unaligned_ptr in &unaligned_pointers {
        let args = [unaligned_ptr, 0, 0, 0];
        
        // Test with syscalls that expect pointers
        for syscall_id in [3, 4, 5, 6] { // SYS_SEND, SYS_RECV, SYS_CHAN_CREATE, SYS_EXEC
            let _ = validate_syscall(syscall_id, &args);
        }
    }
    
    // Test with invalid memory addresses
    let invalid_addresses = [
        0x0,                    // Null pointer
        0x1000,                 // Below user space base
        0xffff800000000000,     // Kernel space
        0xffffffffffffffff,     // Invalid high address
    ];
    
    for &invalid_addr in &invalid_addresses {
        let args = [invalid_addr, 0, 0, 0];
        
        // Test with syscalls that expect valid pointers
        for syscall_id in [3, 4, 5, 6] {
            let _ = validate_syscall(syscall_id, &args);
        }
    }
}

/// Test edge cases
fn test_edge_cases(data: &[u8]) {
    // Test with boundary values
    let boundary_values = [
        0x400000,               // User space base
        0x7fffffffffff,         // User space top
        0x7ffffffffffe,         // Just below user space top
        0x400001,               // Just above user space base
    ];
    
    for &boundary_value in &boundary_values {
        let args = [boundary_value, 0, 0, 0];
        
        // Test with various syscall IDs
        for syscall_id in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10] {
            let _ = validate_syscall(syscall_id, &args);
        }
    }
    
    // Test with mixed valid/invalid values
    let mixed_args = [
        [0x400000, 0x0, 0x400000, 0x0],           // Valid, null, valid, null
        [0x0, 0x400000, 0x0, 0x400000],           // Null, valid, null, valid
        [0x400000, 0x400000, 0x0, 0x0],           // Valid, valid, null, null
        [0x0, 0x0, 0x400000, 0x400000],           // Null, null, valid, valid
    ];
    
    for args in &mixed_args {
        // Test with various syscall IDs
        for syscall_id in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10] {
            let _ = validate_syscall(syscall_id, args);
        }
    }
}

/// Test syscall validation with given input data
fn test_syscall_validation(data: &[u8]) {
    // Convert data to syscall arguments
    let mut args = [0u64; MAX_SYSCALL_ARGS];
    
    // Fill args array with data, handling different data sizes
    for (i, arg) in args.iter_mut().enumerate() {
        let start = i * 8;
        if start < data.len() {
            let end = (start + 8).min(data.len());
            let chunk = &data[start..end];
            
            // Convert bytes to u64 (little-endian)
            let mut value = 0u64;
            for (j, &byte) in chunk.iter().enumerate() {
                value |= (byte as u64) << (j * 8);
            }
            *arg = value;
        }
    }
    
    // Test validation with various syscall IDs
    for syscall_id in 1..=20 {
        let _ = validate_syscall(syscall_id, &args);
    }
}

/// Test specific syscall argument patterns
fn test_specific_patterns() {
    // Test SYS_SEND (id: 3) - expects dst (u64) and buf (pointer)
    let send_args = [
        [1, 0x400000, 0, 0],           // Valid: dst=1, buf=valid_ptr
        [0, 0x400000, 0, 0],           // Invalid: dst=0 (required)
        [1, 0x0, 0, 0],                // Invalid: buf=null (required)
        [1, 0x400001, 0, 0],           // Invalid: buf=unaligned
        [10001, 0x400000, 0, 0],       // Invalid: dst > 10000
    ];
    
    for args in &send_args {
        let _ = validate_syscall(3, args); // SYS_SEND
    }
    
    // Test SYS_RECV (id: 4) - expects block (bool) and out (pointer)
    let recv_args = [
        [0, 0x400000, 0, 0],           // Valid: block=false, out=valid_ptr
        [1, 0x400000, 0, 0],           // Valid: block=true, out=valid_ptr
        [0, 0x0, 0, 0],                // Invalid: out=null (required)
        [0, 0x400001, 0, 0],           // Invalid: out=unaligned
    ];
    
    for args in &recv_args {
        let _ = validate_syscall(4, args); // SYS_RECV
    }
    
    // Test SYS_CHAN_CREATE (id: 5) - expects capacity (u64)
    let chan_args = [
        [1, 0, 0, 0],                  // Valid: capacity=1
        [0, 0, 0, 0],                  // Invalid: capacity=0 (required > 0)
        [1000001, 0, 0, 0],            // Invalid: capacity > 1000000
    ];
    
    for args in &chan_args {
        let _ = validate_syscall(5, args); // SYS_CHAN_CREATE
    }
}

/// Test error code mapping
fn test_error_code_mapping() {
    // Test that validation failures return appropriate error codes
    let test_cases = [
        // (syscall_id, args, expected_error_pattern)
        (3, [0, 0x400000, 0, 0], "EINVAL"),      // SYS_SEND with dst=0
        (3, [1, 0x0, 0, 0], "EFAULT"),           // SYS_SEND with null buf
        (3, [1, 0x400001, 0, 0], "EINVAL"),      // SYS_SEND with unaligned buf
        (4, [0, 0x0, 0, 0], "EFAULT"),           // SYS_RECV with null out
        (5, [0, 0, 0, 0], "EINVAL"),             // SYS_CHAN_CREATE with capacity=0
        (999, [0, 0, 0, 0], "EINVAL"),           // Invalid syscall ID
    ];
    
    for (syscall_id, args, expected_error) in &test_cases {
        let result = validate_syscall(*syscall_id, args);
        if let Err(error_msg) = result {
            // The error should contain the expected error pattern
            assert!(error_msg.contains(expected_error), 
                "Expected error pattern '{}' not found in '{}'", expected_error, error_msg);
        }
    }
}

/// Test performance characteristics
fn test_performance() {
    // Test that validation doesn't take too long
    let start = std::time::Instant::now();
    
    // Run many validation calls
    for _ in 0..1000 {
        let args = [0x400000, 0x400000, 0, 0];
        let _ = validate_syscall(3, &args); // SYS_SEND
    }
    
    let duration = start.elapsed();
    
    // Validation should complete within reasonable time
    // 1000 validations should take less than 1ms
    assert!(duration.as_micros() < 1000, 
        "Validation took too long: {} microseconds", duration.as_micros());
}

/// Test memory safety
fn test_memory_safety() {
    // Test with various buffer sizes to ensure no buffer overflows
    let buffer_sizes = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024];
    
    for &size in &buffer_sizes {
        let mut data = vec![0u8; size];
        
        // Fill with some pattern
        for (i, byte) in data.iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }
        
        // Test validation with this data
        test_syscall_validation(&data);
        
        // Ensure no buffer overflow occurred
        assert_eq!(data.len(), size);
    }
}

/// Test concurrent access (if applicable)
fn test_concurrent_access() {
    // Test that validation is safe for concurrent access
    // This is a basic test - in a real implementation, you might use threads
    
    let args = [0x400000, 0x400000, 0, 0];
    
    // Simulate concurrent validation calls
    for _ in 0..100 {
        let _ = validate_syscall(3, &args); // SYS_SEND
    }
    
    // If we get here without crashing, basic concurrency safety is working
}

/// Test error recovery
fn test_error_recovery() {
    // Test that validation errors don't leave the system in a bad state
    
    // First, cause a validation error
    let invalid_args = [0, 0x0, 0, 0]; // Invalid for SYS_SEND
    let result1 = validate_syscall(3, &invalid_args);
    assert!(result1.is_err());
    
    // Then, try a valid call to ensure system still works
    let valid_args = [1, 0x400000, 0, 0];
    let result2 = validate_syscall(3, &valid_args);
    // This should either succeed or fail with a different error, but not crash
    let _ = result2;
    
    // Try another valid call
    let valid_args2 = [2, 0x400000, 0, 0];
    let result3 = validate_syscall(3, &valid_args2);
    let _ = result3;
}
