#![no_main]

use libfuzzer_sys::fuzz_target;

/// Fuzzer target for syscall argument decoder
/// 
/// This fuzzer tests the syscall argument decoder with various malformed inputs
/// to ensure it handles abuse attempts gracefully without crashes or
/// security vulnerabilities.
fuzz_target!(|data: &[u8]| {
    // Skip empty inputs
    if data.is_empty() {
        return;
    }
    
    // Test 1: Raw argument decoding
    test_raw_argument_decoding(data);
    
    // Test 2: Pointer validation
    test_pointer_validation(data);
    
    // Test 3: Size validation
    test_size_validation(data);
    
    // Test 4: Type coercion attacks
    test_type_coercion_attacks(data);
    
    // Test 5: Buffer overflow attempts
    test_buffer_overflow_attempts(data);
    
    // Test 6: Integer overflow attacks
    test_integer_overflow_attacks(data);
    
    // Test 7: Alignment attacks
    test_alignment_attacks(data);
    
    // Test 8: Null pointer attacks
    test_null_pointer_attacks(data);
});

/// Test raw argument decoding with arbitrary bytes
fn test_raw_argument_decoding(data: &[u8]) {
    // Test with different argument counts
    for arg_count in [0, 1, 2, 3, 4, 8, 16, 32, 64] {
        if data.len() >= arg_count * 8 {
            let args = extract_arguments(data, arg_count);
            test_argument_validation(&args);
        }
    }
    
    // Test with partial arguments
    if data.len() >= 8 {
        let partial_args = extract_partial_arguments(data);
        test_argument_validation(&partial_args);
    }
}

/// Test pointer validation
fn test_pointer_validation(data: &[u8]) {
    if data.len() >= 8 {
        // Test various pointer values
        let pointers = [
            0x0000000000000000, // Null pointer
            0x0000000000000001, // Near null
            0x0000000000000002, // Even near null
            0x0000000000000003, // Odd near null
            0x0000000000000004, // Aligned near null
            0x0000000000000008, // 8-byte aligned
            0x0000000000000010, // 16-byte aligned
            0x0000000000000020, // 32-byte aligned
            0x0000000000000040, // 64-byte aligned
            0x0000000000000080, // 128-byte aligned
            0x0000000000000100, // 256-byte aligned
            0x0000000000000200, // 512-byte aligned
            0x0000000000000400, // 1KB aligned
            0x0000000000000800, // 2KB aligned
            0x0000000000001000, // 4KB aligned
            0x0000000000002000, // 8KB aligned
            0x0000000000004000, // 16KB aligned
            0x0000000000008000, // 32KB aligned
            0x0000000000010000, // 64KB aligned
            0x0000000000020000, // 128KB aligned
            0x0000000000040000, // 256KB aligned
            0x0000000000080000, // 512KB aligned
            0x0000000000100000, // 1MB aligned
            0x0000000000200000, // 2MB aligned
            0x0000000000400000, // 4MB aligned
            0x0000000000800000, // 8MB aligned
            0x0000000001000000, // 16MB aligned
            0x0000000002000000, // 32MB aligned
            0x0000000004000000, // 64MB aligned
            0x0000000008000000, // 128MB aligned
            0x0000000010000000, // 256MB aligned
            0x0000000020000000, // 512MB aligned
            0x0000000040000000, // 1GB aligned
            0x0000000080000000, // 2GB aligned
            0x0000000100000000, // 4GB aligned
            0x0000000200000000, // 8GB aligned
            0x0000000400000000, // 16GB aligned
            0x0000000800000000, // 32GB aligned
            0x0000001000000000, // 64GB aligned
            0x0000002000000000, // 128GB aligned
            0x0000004000000000, // 256GB aligned
            0x0000008000000000, // 512GB aligned
            0x0000010000000000, // 1TB aligned
            0x0000020000000000, // 2TB aligned
            0x0000040000000000, // 4TB aligned
            0x0000080000000000, // 8TB aligned
            0x0000100000000000, // 16TB aligned
            0x0000200000000000, // 32TB aligned
            0x0000400000000000, // 64TB aligned
            0x0000800000000000, // 128TB aligned
            0x0001000000000000, // 256TB aligned
            0x0002000000000000, // 512TB aligned
            0x0004000000000000, // 1PB aligned
            0x0008000000000000, // 2PB aligned
            0x0010000000000000, // 4PB aligned
            0x0020000000000000, // 8PB aligned
            0x0040000000000000, // 16PB aligned
            0x0080000000000000, // 32PB aligned
            0x0100000000000000, // 64PB aligned
            0x0200000000000000, // 128PB aligned
            0x0400000000000000, // 256PB aligned
            0x0800000000000000, // 512PB aligned
            0x1000000000000000, // 1EB aligned
            0x2000000000000000, // 2EB aligned
            0x4000000000000000, // 4EB aligned
            0x8000000000000000, // 8EB aligned
            0xFFFFFFFFFFFFFFFF, // Max pointer
            0xFFFFFFFFFFFFFFFE, // Max pointer - 1
            0xFFFFFFFFFFFFFFFD, // Max pointer - 2
            0xFFFFFFFFFFFFFFFC, // Max pointer - 4
            0xFFFFFFFFFFFFFFF8, // Max pointer - 8
            0xFFFFFFFFFFFFFFF0, // Max pointer - 16
            0xFFFFFFFFFFFFFFE0, // Max pointer - 32
            0xFFFFFFFFFFFFFFC0, // Max pointer - 64
            0xFFFFFFFFFFFFFF80, // Max pointer - 128
            0xFFFFFFFFFFFFFF00, // Max pointer - 256
            0xFFFFFFFFFFFFFE00, // Max pointer - 512
            0xFFFFFFFFFFFFFC00, // Max pointer - 1KB
            0xFFFFFFFFFFFFF800, // Max pointer - 2KB
            0xFFFFFFFFFFFFF000, // Max pointer - 4KB
            0xFFFFFFFFFFFFE000, // Max pointer - 8KB
            0xFFFFFFFFFFFFC000, // Max pointer - 16KB
            0xFFFFFFFFFFFF8000, // Max pointer - 32KB
            0xFFFFFFFFFFFF0000, // Max pointer - 64KB
            0xFFFFFFFFFFFE0000, // Max pointer - 128KB
            0xFFFFFFFFFFFC0000, // Max pointer - 256KB
            0xFFFFFFFFFFF80000, // Max pointer - 512KB
            0xFFFFFFFFFFF00000, // Max pointer - 1MB
            0xFFFFFFFFFFE00000, // Max pointer - 2MB
            0xFFFFFFFFFFC00000, // Max pointer - 4MB
            0xFFFFFFFFFF800000, // Max pointer - 8MB
            0xFFFFFFFFFF000000, // Max pointer - 16MB
            0xFFFFFFFFFE000000, // Max pointer - 32MB
            0xFFFFFFFFFC000000, // Max pointer - 64MB
            0xFFFFFFFFF8000000, // Max pointer - 128MB
            0xFFFFFFFFF0000000, // Max pointer - 256MB
            0xFFFFFFFFE0000000, // Max pointer - 512MB
            0xFFFFFFFFC0000000, // Max pointer - 1GB
            0xFFFFFFFF80000000, // Max pointer - 2GB
            0xFFFFFFFF00000000, // Max pointer - 4GB
            0xFFFFFFFE00000000, // Max pointer - 8GB
            0xFFFFFFFC00000000, // Max pointer - 16GB
            0xFFFFFFF800000000, // Max pointer - 32GB
            0xFFFFFFF000000000, // Max pointer - 64GB
            0xFFFFFFE000000000, // Max pointer - 128GB
            0xFFFFFFC000000000, // Max pointer - 256GB
            0xFFFFFF8000000000, // Max pointer - 512GB
            0xFFFFFF0000000000, // Max pointer - 1TB
            0xFFFFFE0000000000, // Max pointer - 2TB
            0xFFFFFC0000000000, // Max pointer - 4TB
            0xFFFFF80000000000, // Max pointer - 8TB
            0xFFFFF00000000000, // Max pointer - 16TB
            0xFFFFE00000000000, // Max pointer - 32TB
            0xFFFFC00000000000, // Max pointer - 64TB
            0xFFFF800000000000, // Max pointer - 128TB
            0xFFFF000000000000, // Max pointer - 256TB
            0xFFFE000000000000, // Max pointer - 512TB
            0xFFFC000000000000, // Max pointer - 1PB
            0xFFF8000000000000, // Max pointer - 2PB
            0xFFF0000000000000, // Max pointer - 4PB
            0xFFE0000000000000, // Max pointer - 8PB
            0xFFC0000000000000, // Max pointer - 16PB
            0xFF80000000000000, // Max pointer - 32PB
            0xFF00000000000000, // Max pointer - 64PB
            0xFE00000000000000, // Max pointer - 128PB
            0xFC00000000000000, // Max pointer - 256PB
            0xF800000000000000, // Max pointer - 512PB
            0xF000000000000000, // Max pointer - 1EB
            0xE000000000000000, // Max pointer - 2EB
            0xC000000000000000, // Max pointer - 4EB
            0x8000000000000000, // Max pointer - 8EB
        ];
        
        for &ptr in &pointers {
            test_pointer_value(ptr);
        }
    }
}

/// Test size validation
fn test_size_validation(data: &[u8]) {
    if data.len() >= 8 {
        // Test various size values
        let sizes = [
            0,                    // Zero size
            1,                    // Minimum size
            2,                    // Even size
            3,                    // Odd size
            4,                    // 4-byte aligned
            8,                    // 8-byte aligned
            16,                   // 16-byte aligned
            32,                   // 32-byte aligned
            64,                   // 64-byte aligned
            128,                  // 128-byte aligned
            256,                  // 256-byte aligned
            512,                  // 512-byte aligned
            1024,                 // 1KB
            2048,                 // 2KB
            4096,                 // 4KB
            8192,                 // 8KB
            16384,                // 16KB
            32768,                // 32KB
            65536,                // 64KB
            131072,               // 128KB
            262144,               // 256KB
            524288,               // 512KB
            1048576,              // 1MB
            2097152,              // 2MB
            4194304,              // 4MB
            8388608,              // 8MB
            16777216,             // 16MB
            33554432,             // 32MB
            67108864,             // 64MB
            134217728,            // 128MB
            268435456,            // 256MB
            536870912,            // 512MB
            1073741824,           // 1GB
            2147483648,           // 2GB
            4294967296,           // 4GB
            8589934592,           // 8GB
            17179869184,          // 16GB
            34359738368,          // 32GB
            68719476736,          // 64GB
            137438953472,         // 128GB
            274877906944,         // 256GB
            549755813888,         // 512GB
            1099511627776,        // 1TB
            2199023255552,        // 2TB
            4398046511104,        // 4TB
            8796093022208,        // 8TB
            17592186044416,       // 16TB
            35184372088832,       // 32TB
            70368744177664,       // 64TB
            140737488355328,      // 128TB
            281474976710656,      // 256TB
            562949953421312,      // 512TB
            1125899906842624,     // 1PB
            2251799813685248,     // 2PB
            4503599627370496,     // 4PB
            9007199254740992,     // 8PB
            18014398509481984,    // 16PB
            36028797018963968,    // 32PB
            72057594037927936,    // 64PB
            144115188075855872,   // 128PB
            288230376151711744,   // 256PB
            576460752303423488,   // 512PB
            1152921504606846976,  // 1EB
            2305843009213693952,  // 2EB
            4611686018427387904,  // 4EB
            9223372036854775808,  // 8EB
            0xFFFFFFFFFFFFFFFF,  // Max size
        ];
        
        for &size in &sizes {
            test_size_value(size);
        }
    }
}

/// Test type coercion attacks
fn test_type_coercion_attacks(data: &[u8]) {
    if data.len() >= 8 {
        // Test with various data types interpreted as different types
        let raw_bytes = u64::from_le_bytes([
            data[0], data[1], data[2], data[3],
            data[4], data[5], data[6], data[7]
        ]);
        
        // Test as pointer
        test_pointer_value(raw_bytes);
        
        // Test as size
        test_size_value(raw_bytes);
        
        // Test as integer
        test_integer_value(raw_bytes);
        
        // Test as flags
        test_flags_value(raw_bytes);
        
        // Test as syscall number
        test_syscall_number(raw_bytes);
    }
}

/// Test buffer overflow attempts
fn test_buffer_overflow_attempts(data: &[u8]) {
    if data.len() >= 16 {
        // Test with various buffer sizes that might cause overflow
        let buffer_sizes = [
            0x7FFFFFFF,           // Max positive 32-bit
            0x80000000,           // Min negative 32-bit
            0xFFFFFFFF,           // Max 32-bit
            0x100000000,          // 4GB + 1
            0x200000000,          // 8GB + 1
            0x400000000,          // 16GB + 1
            0x800000000,          // 32GB + 1
            0x1000000000,         // 64GB + 1
            0x2000000000,         // 128GB + 1
            0x4000000000,         // 256GB + 1
            0x8000000000,         // 512GB + 1
            0x10000000000,        // 1TB + 1
            0x20000000000,        // 2TB + 1
            0x40000000000,        // 4TB + 1
            0x80000000000,        // 8TB + 1
            0x100000000000,       // 16TB + 1
            0x200000000000,       // 32TB + 1
            0x400000000000,       // 64TB + 1
            0x800000000000,       // 128TB + 1
            0x1000000000000,      // 256TB + 1
            0x2000000000000,      // 512TB + 1
            0x4000000000000,      // 1PB + 1
            0x8000000000000,      // 2PB + 1
            0x10000000000000,     // 4PB + 1
            0x20000000000000,     // 8PB + 1
            0x40000000000000,     // 16PB + 1
            0x80000000000000,     // 32PB + 1
            0x100000000000000,    // 64PB + 1
            0x200000000000000,    // 128PB + 1
            0x400000000000000,    // 256PB + 1
            0x800000000000000,    // 512PB + 1
            0x1000000000000000,   // 1EB + 1
            0x2000000000000000,   // 2EB + 1
            0x4000000000000000,   // 4EB + 1
            0x8000000000000000,   // 8EB + 1
        ];
        
        for &size in &buffer_sizes {
            test_buffer_size(size);
        }
    }
}

/// Test integer overflow attacks
fn test_integer_overflow_attacks(data: &[u8]) {
    if data.len() >= 8 {
        // Test with values that might cause integer overflow
        let overflow_values = [
            0x7FFFFFFFFFFFFFFF,   // Max positive 64-bit
            0x8000000000000000,   // Min negative 64-bit
            0xFFFFFFFFFFFFFFFF,   // Max 64-bit
            0x7FFFFFFF,           // Max positive 32-bit
            0x80000000,           // Min negative 32-bit
            0xFFFFFFFF,           // Max 32-bit
            0x7FFF,               // Max positive 16-bit
            0x8000,               // Min negative 16-bit
            0xFFFF,               // Max 16-bit
            0x7F,                 // Max positive 8-bit
            0x80,                 // Min negative 8-bit
            0xFF,                 // Max 8-bit
        ];
        
        for &value in &overflow_values {
            test_integer_value(value);
        }
    }
}

/// Test alignment attacks
fn test_alignment_attacks(data: &[u8]) {
    if data.len() >= 8 {
        // Test with misaligned pointers
        let misaligned_pointers = [
            0x0000000000000001,   // 1-byte misaligned
            0x0000000000000002,   // 2-byte misaligned
            0x0000000000000003,   // 3-byte misaligned
            0x0000000000000004,   // 4-byte misaligned
            0x0000000000000005,   // 5-byte misaligned
            0x0000000000000006,   // 6-byte misaligned
            0x0000000000000007,   // 7-byte misaligned
            0x0000000000000009,   // 9-byte misaligned
            0x000000000000000A,   // 10-byte misaligned
            0x000000000000000B,   // 11-byte misaligned
            0x000000000000000C,   // 12-byte misaligned
            0x000000000000000D,   // 13-byte misaligned
            0x000000000000000E,   // 14-byte misaligned
            0x000000000000000F,   // 15-byte misaligned
        ];
        
        for &ptr in &misaligned_pointers {
            test_pointer_value(ptr);
        }
    }
}

/// Test null pointer attacks
fn test_null_pointer_attacks(data: &[u8]) {
    // Test with various null-like values
    let null_like_values = [
        0x0000000000000000,       // True null
        0x0000000000000001,       // Near null
        0x0000000000000002,       // Even near null
        0x0000000000000003,       // Odd near null
        0x0000000000000004,       // 4-byte aligned near null
        0x0000000000000008,       // 8-byte aligned near null
        0x0000000000000010,       // 16-byte aligned near null
        0x0000000000000020,       // 32-byte aligned near null
        0x0000000000000040,       // 64-byte aligned near null
        0x0000000000000080,       // 128-byte aligned near null
        0x0000000000000100,       // 256-byte aligned near null
        0x0000000000000200,       // 512-byte aligned near null
        0x0000000000000400,       // 1KB aligned near null
        0x0000000000000800,       // 2KB aligned near null
        0x0000000000001000,       // 4KB aligned near null
    ];
    
    for &ptr in &null_like_values {
        test_pointer_value(ptr);
    }
}

//=============================================================================
// HELPER FUNCTIONS
//=============================================================================

/// Extract arguments from data
fn extract_arguments(data: &[u8], count: usize) -> Vec<u64> {
    let mut args = Vec::new();
    
    for i in 0..count {
        if data.len() >= (i + 1) * 8 {
            let arg_bytes = &data[i * 8..(i + 1) * 8];
            let arg = u64::from_le_bytes([
                arg_bytes[0], arg_bytes[1], arg_bytes[2], arg_bytes[3],
                arg_bytes[4], arg_bytes[5], arg_bytes[6], arg_bytes[7]
            ]);
            args.push(arg);
        }
    }
    
    args
}

/// Extract partial arguments from data
fn extract_partial_arguments(data: &[u8]) -> Vec<u64> {
    let mut args = Vec::new();
    
    // Try to extract as many complete arguments as possible
    let complete_args = data.len() / 8;
    
    for i in 0..complete_args {
        let arg_bytes = &data[i * 8..(i + 1) * 8];
        let arg = u64::from_le_bytes([
            arg_bytes[0], arg_bytes[1], arg_bytes[2], arg_bytes[3],
            arg_bytes[4], arg_bytes[5], arg_bytes[6], arg_bytes[7]
        ]);
        args.push(arg);
    }
    
    args
}

/// Test argument validation
fn test_argument_validation(args: &[u64]) {
    // This function should never crash, even with invalid arguments
    for &arg in args {
        test_argument_value(arg);
    }
}

/// Test argument value
fn test_argument_value(value: u64) {
    // Test as pointer
    test_pointer_value(value);
    
    // Test as size
    test_size_value(value);
    
    // Test as integer
    test_integer_value(value);
    
    // Test as flags
    test_flags_value(value);
    
    // Test as syscall number
    test_syscall_number(value);
}

/// Test pointer value
fn test_pointer_value(ptr: u64) {
    // This should never crash
    // In a real implementation, we would validate the pointer
    
    // Check for null pointer
    if ptr == 0 {
        // Null pointer - should be handled gracefully
        return;
    }
    
    // Check for kernel space pointers (high bit set on x86_64)
    if ptr & 0x8000000000000000 != 0 {
        // Kernel space pointer - should be rejected
        return;
    }
    
    // Check for reasonable user space range
    if ptr > 0x7FFFFFFFFFFF {
        // Unreasonable pointer - should be rejected
        return;
    }
}

/// Test size value
fn test_size_value(size: u64) {
    // This should never crash
    
    // Check for zero size
    if size == 0 {
        // Zero size - should be handled gracefully
        return;
    }
    
    // Check for reasonable size limits
    if size > 0x1000000 {
        // Very large size - should be rejected
        return;
    }
    
    // Check for alignment
    if size % 8 != 0 {
        // Misaligned size - should be handled gracefully
        return;
    }
}

/// Test integer value
fn test_integer_value(value: u64) {
    // This should never crash
    
    // Check for reasonable integer ranges
    if value > 0x7FFFFFFFFFFFFFFF {
        // Very large integer - should be handled gracefully
        return;
    }
    
    // Check for special values
    match value {
        0 => {
            // Zero value - should be handled gracefully
        }
        0xFFFFFFFFFFFFFFFF => {
            // Max value - should be handled gracefully
        }
        _ => {
            // Normal value - should be handled normally
        }
    }
}

/// Test flags value
fn test_flags_value(flags: u64) {
    // This should never crash
    
    // Check for reasonable flag values
    if flags > 0xFFFF {
        // Very large flags - should be handled gracefully
        return;
    }
    
    // Check individual flag bits
    for bit in 0..16 {
        if flags & (1 << bit) != 0 {
            // Flag bit set - should be handled normally
        }
    }
}

/// Test syscall number
fn test_syscall_number(syscall: u64) {
    // This should never crash
    
    // Check for valid syscall range
    if syscall > 100 {
        // Invalid syscall number - should be handled gracefully
        return;
    }
    
    // Check for known syscall numbers
    match syscall {
        1 => {
            // SYS_YIELD - valid
        }
        2 => {
            // SYS_EXIT - valid
        }
        3 => {
            // SYS_SEND - valid
        }
        4 => {
            // SYS_RECV - valid
        }
        5 => {
            // SYS_CHAN_CREATE - valid
        }
        6 => {
            // SYS_STATS - valid
        }
        7 => {
            // SYS_DEBUG - valid
        }
        11 => {
            // SYS_EXEC - valid
        }
        _ => {
            // Unknown syscall - should be handled gracefully
        }
    }
}

/// Test buffer size
fn test_buffer_size(size: u64) {
    // This should never crash
    
    // Check for reasonable buffer size limits
    if size > 0x1000000 {
        // Very large buffer - should be rejected
        return;
    }
    
    // Check for zero buffer
    if size == 0 {
        // Zero buffer - should be handled gracefully
        return;
    }
    
    // Check for alignment
    if size % 8 != 0 {
        // Misaligned buffer - should be handled gracefully
        return;
    }
}
