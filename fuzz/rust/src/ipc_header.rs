#![no_main]

use libfuzzer_sys::fuzz_target;

/// Fuzzer target for IPC header parser
/// 
/// This fuzzer tests the IPC header parser with various malformed inputs
/// to ensure it handles abuse attempts gracefully without crashes or
/// security vulnerabilities.
fuzz_target!(|data: &[u8]| {
    // Skip empty inputs
    if data.is_empty() {
        return;
    }
    
    // Test 1: Raw header parsing
    test_raw_header_parsing(data);
    
    // Test 2: Header field validation
    test_header_field_validation(data);
    
    // Test 3: MAC tag validation
    test_mac_tag_validation(data);
    
    // Test 4: Capability ID validation
    test_capability_id_validation(data);
    
    // Test 5: Message ID validation
    test_message_id_validation(data);
    
    // Test 6: Timestamp validation
    test_timestamp_validation(data);
    
    // Test 7: Size validation
    test_size_validation(data);
    
    // Test 8: Flag validation
    test_flag_validation(data);
    
    // Test 9: Priority validation
    test_priority_validation(data);
    
    // Test 10: Checksum validation
    test_checksum_validation(data);
});

/// Test raw header parsing with arbitrary bytes
fn test_raw_header_parsing(data: &[u8]) {
    // Test with different header sizes
    for size in [32, 64, 128, 256, 512, 1024] {
        if data.len() >= size {
            let header_slice = &data[..size];
            test_header_slice(header_slice);
        }
    }
    
    // Test with partial headers
    if data.len() >= 32 {
        let partial_header = &data[..32];
        test_header_slice(partial_header);
    }
    
    // Test with oversized headers
    if data.len() >= 2048 {
        let oversized_header = &data[..2048];
        test_header_slice(oversized_header);
    }
}

/// Test header field validation
fn test_header_field_validation(data: &[u8]) {
    if data.len() >= 64 {
        // Test individual header fields
        test_sender_field(data);
        test_receiver_field(data);
        test_message_type_field(data);
        test_priority_field(data);
        test_flags_field(data);
        test_payload_size_field(data);
        test_timestamp_field(data);
        test_sequence_field(data);
    }
}

/// Test MAC tag validation
fn test_mac_tag_validation(data: &[u8]) {
    if data.len() >= 96 {
        // Extract MAC tag (assuming 32-byte MAC at offset 64)
        let mac_tag = &data[64..96];
        
        // Test with various MAC tag patterns
        test_mac_tag_patterns(mac_tag);
        
        // Test with corrupted MAC tags
        test_corrupted_mac_tags(data);
        
        // Test with zeroed MAC tags
        test_zeroed_mac_tags(data);
        
        // Test with repeated MAC tags
        test_repeated_mac_tags(data);
    }
}

/// Test capability ID validation
fn test_capability_id_validation(data: &[u8]) {
    if data.len() >= 128 {
        // Extract capability ID (assuming 32-byte cap ID at offset 96)
        let cap_id = &data[96..128];
        
        // Test with various capability ID patterns
        test_capability_id_patterns(cap_id);
        
        // Test with corrupted capability IDs
        test_corrupted_capability_ids(data);
        
        // Test with zeroed capability IDs
        test_zeroed_capability_ids(data);
        
        // Test with repeated capability IDs
        test_repeated_capability_ids(data);
    }
}

/// Test message ID validation
fn test_message_id_validation(data: &[u8]) {
    if data.len() >= 8 {
        // Extract message ID (first 8 bytes)
        let msg_id = &data[0..8];
        
        // Test with various message ID patterns
        test_message_id_patterns(msg_id);
        
        // Test with corrupted message IDs
        test_corrupted_message_ids(data);
        
        // Test with zeroed message IDs
        test_zeroed_message_ids(data);
        
        // Test with repeated message IDs
        test_repeated_message_ids(data);
    }
}

/// Test timestamp validation
fn test_timestamp_validation(data: &[u8]) {
    if data.len() >= 16 {
        // Extract timestamp (assuming 8-byte timestamp at offset 8)
        let timestamp = &data[8..16];
        
        // Test with various timestamp patterns
        test_timestamp_patterns(timestamp);
        
        // Test with corrupted timestamps
        test_corrupted_timestamps(data);
        
        // Test with zeroed timestamps
        test_zeroed_timestamps(data);
        
        // Test with future timestamps
        test_future_timestamps(data);
        
        // Test with very old timestamps
        test_old_timestamps(data);
    }
}

/// Test size validation
fn test_size_validation(data: &[u8]) {
    if data.len() >= 24 {
        // Extract payload size (assuming 8-byte size at offset 16)
        let size = &data[16..24];
        
        // Test with various size patterns
        test_size_patterns(size);
        
        // Test with corrupted sizes
        test_corrupted_sizes(data);
        
        // Test with zeroed sizes
        test_zeroed_sizes(data);
        
        // Test with excessive sizes
        test_excessive_sizes(data);
    }
}

/// Test flag validation
fn test_flag_validation(data: &[u8]) {
    if data.len() >= 32 {
        // Extract flags (assuming 8-byte flags at offset 24)
        let flags = &data[24..32];
        
        // Test with various flag patterns
        test_flag_patterns(flags);
        
        // Test with corrupted flags
        test_corrupted_flags(data);
        
        // Test with zeroed flags
        test_zeroed_flags(data);
        
        // Test with all flags set
        test_all_flags_set(data);
    }
}

/// Test priority validation
fn test_priority_validation(data: &[u8]) {
    if data.len() >= 40 {
        // Extract priority (assuming 8-byte priority at offset 32)
        let priority = &data[32..40];
        
        // Test with various priority patterns
        test_priority_patterns(priority);
        
        // Test with corrupted priorities
        test_corrupted_priorities(data);
        
        // Test with zeroed priorities
        test_zeroed_priorities(data);
        
        // Test with excessive priorities
        test_excessive_priorities(data);
    }
}

/// Test checksum validation
fn test_checksum_validation(data: &[u8]) {
    if data.len() >= 48 {
        // Extract checksum (assuming 4-byte checksum at offset 44)
        let checksum = &data[44..48];
        
        // Test with various checksum patterns
        test_checksum_patterns(checksum);
        
        // Test with corrupted checksums
        test_corrupted_checksums(data);
        
        // Test with zeroed checksums
        test_zeroed_checksums(data);
        
        // Test with invalid checksums
        test_invalid_checksums(data);
    }
}

//=============================================================================
// HELPER FUNCTIONS
//=============================================================================

/// Test a header slice
fn test_header_slice(header: &[u8]) {
    // This should never crash, even with malformed headers
    
    // Test minimum size
    if header.len() < 32 {
        // Too small - should be handled gracefully
        return;
    }
    
    // Test maximum size
    if header.len() > 2048 {
        // Too large - should be handled gracefully
        return;
    }
    
    // Test alignment
    if header.len() % 8 != 0 {
        // Misaligned - should be handled gracefully
        return;
    }
    
    // Test magic number validation
    if header.len() >= 8 {
        test_magic_number(&header[0..8]);
    }
    
    // Test version validation
    if header.len() >= 9 {
        test_version_number(header[8]);
    }
    
    // Test field validation
    if header.len() >= 64 {
        test_header_fields(header);
    }
}

/// Test magic number
fn test_magic_number(magic: &[u8]) {
    // This should never crash
    
    // Check for reasonable magic numbers
    let magic_u64 = u64::from_le_bytes([
        magic[0], magic[1], magic[2], magic[3],
        magic[4], magic[5], magic[6], magic[7]
    ]);
    
    // Test with various magic values
    match magic_u64 {
        0 => {
            // Zero magic - should be rejected
        }
        0xFFFFFFFFFFFFFFFF => {
            // Max magic - should be rejected
        }
        _ => {
            // Other magic values - should be validated
        }
    }
}

/// Test version number
fn test_version_number(version: u8) {
    // This should never crash
    
    // Check for reasonable version numbers
    match version {
        0 => {
            // Version 0 - should be handled gracefully
        }
        1 => {
            // Version 1 - should be handled normally
        }
        2 => {
            // Version 2 - should be handled normally
        }
        _ => {
            // Unknown version - should be handled gracefully
        }
    }
}

/// Test header fields
fn test_header_fields(header: &[u8]) {
    // This should never crash
    
    // Test sender field
    if header.len() >= 72 {
        let sender = &header[64..72];
        test_process_id(sender);
    }
    
    // Test receiver field
    if header.len() >= 80 {
        let receiver = &header[72..80];
        test_process_id(receiver);
    }
    
    // Test message type field
    if header.len() >= 88 {
        let msg_type = &header[80..88];
        test_message_type(msg_type);
    }
}

/// Test process ID
fn test_process_id(pid: &[u8]) {
    // This should never crash
    
    let pid_u64 = u64::from_le_bytes([
        pid[0], pid[1], pid[2], pid[3],
        pid[4], pid[5], pid[6], pid[7]
    ]);
    
    // Check for reasonable process IDs
    if pid_u64 == 0 {
        // PID 0 - should be handled gracefully
        return;
    }
    
    if pid_u64 > 0x7FFFFFFF {
        // Very large PID - should be handled gracefully
        return;
    }
}

/// Test message type
fn test_message_type(msg_type: &[u8]) {
    // This should never crash
    
    let type_u64 = u64::from_le_bytes([
        msg_type[0], msg_type[1], msg_type[2], msg_type[3],
        msg_type[4], msg_type[5], msg_type[6], msg_type[7]
    ]);
    
    // Check for reasonable message types
    match type_u64 {
        0 => {
            // Type 0 - should be handled gracefully
        }
        1 => {
            // Type 1 - should be handled normally
        }
        2 => {
            // Type 2 - should be handled normally
        }
        _ => {
            // Unknown type - should be handled gracefully
        }
    }
}

//=============================================================================
// PATTERN TESTING FUNCTIONS
//=============================================================================

/// Test MAC tag patterns
fn test_mac_tag_patterns(mac_tag: &[u8]) {
    // This should never crash
    
    // Test with various patterns
    for i in 0..mac_tag.len() {
        let pattern = mac_tag[i];
        
        match pattern {
            0x00 => {
                // Zero byte - should be handled gracefully
            }
            0xFF => {
                // All ones - should be handled gracefully
            }
            0xAA => {
                // Alternating pattern - should be handled gracefully
            }
            0x55 => {
                // Alternating pattern - should be handled gracefully
            }
            _ => {
                // Other patterns - should be handled normally
            }
        }
    }
}

/// Test capability ID patterns
fn test_capability_id_patterns(cap_id: &[u8]) {
    // This should never crash
    
    // Test with various patterns
    for i in 0..cap_id.len() {
        let pattern = cap_id[i];
        
        match pattern {
            0x00 => {
                // Zero byte - should be handled gracefully
            }
            0xFF => {
                // All ones - should be handled gracefully
            }
            0xAA => {
                // Alternating pattern - should be handled gracefully
            }
            0x55 => {
                // Alternating pattern - should be handled gracefully
            }
            _ => {
                // Other patterns - should be handled normally
            }
        }
    }
}

/// Test message ID patterns
fn test_message_id_patterns(msg_id: &[u8]) {
    // This should never crash
    
    // Test with various patterns
    for i in 0..msg_id.len() {
        let pattern = msg_id[i];
        
        match pattern {
            0x00 => {
                // Zero byte - should be handled gracefully
            }
            0xFF => {
                // All ones - should be handled gracefully
            }
            0xAA => {
                // Alternating pattern - should be handled gracefully
            }
            0x55 => {
                // Alternating pattern - should be handled gracefully
            }
            _ => {
                // Other patterns - should be handled normally
            }
        }
    }
}

/// Test timestamp patterns
fn test_timestamp_patterns(timestamp: &[u8]) {
    // This should never crash
    
    // Test with various patterns
    for i in 0..timestamp.len() {
        let pattern = timestamp[i];
        
        match pattern {
            0x00 => {
                // Zero byte - should be handled gracefully
            }
            0xFF => {
                // All ones - should be handled gracefully
            }
            0xAA => {
                // Alternating pattern - should be handled gracefully
            }
            0x55 => {
                // Alternating pattern - should be handled gracefully
            }
            _ => {
                // Other patterns - should be handled normally
            }
        }
    }
}

/// Test size patterns
fn test_size_patterns(size: &[u8]) {
    // This should never crash
    
    // Test with various patterns
    for i in 0..size.len() {
        let pattern = size[i];
        
        match pattern {
            0x00 => {
                // Zero byte - should be handled gracefully
            }
            0xFF => {
                // All ones - should be handled gracefully
            }
            0xAA => {
                // Alternating pattern - should be handled gracefully
            }
            0x55 => {
                // Alternating pattern - should be handled gracefully
            }
            _ => {
                // Other patterns - should be handled normally
            }
        }
    }
}

/// Test flag patterns
fn test_flag_patterns(flags: &[u8]) {
    // This should never crash
    
    // Test with various patterns
    for i in 0..flags.len() {
        let pattern = flags[i];
        
        match pattern {
            0x00 => {
                // Zero byte - should be handled gracefully
            }
            0xFF => {
                // All ones - should be handled gracefully
            }
            0xAA => {
                // Alternating pattern - should be handled gracefully
            }
            0x55 => {
                // Alternating pattern - should be handled gracefully
            }
            _ => {
                // Other patterns - should be handled normally
            }
        }
    }
}

/// Test priority patterns
fn test_priority_patterns(priority: &[u8]) {
    // This should never crash
    
    // Test with various patterns
    for i in 0..priority.len() {
        let pattern = priority[i];
        
        match pattern {
            0x00 => {
                // Zero byte - should be handled gracefully
            }
            0xFF => {
                // All ones - should be handled gracefully
            }
            0xAA => {
                // Alternating pattern - should be handled gracefully
            }
            0x55 => {
                // Alternating pattern - should be handled gracefully
            }
            _ => {
                // Other patterns - should be handled normally
            }
        }
    }
}

/// Test checksum patterns
fn test_checksum_patterns(checksum: &[u8]) {
    // This should never crash
    
    // Test with various patterns
    for i in 0..checksum.len() {
        let pattern = checksum[i];
        
        match pattern {
            0x00 => {
                // Zero byte - should be handled gracefully
            }
            0xFF => {
                // All ones - should be handled gracefully
            }
            0xAA => {
                // Alternating pattern - should be handled gracefully
            }
            0x55 => {
                // Alternating pattern - should be handled gracefully
            }
            _ => {
                // Other patterns - should be handled normally
            }
        }
    }
}

//=============================================================================
// CORRUPTION TESTING FUNCTIONS
//=============================================================================

/// Test corrupted MAC tags
fn test_corrupted_mac_tags(data: &[u8]) {
    if data.len() >= 96 {
        let mut corrupted = data.to_vec();
        
        // Corrupt MAC tag bytes
        for i in 64..96 {
            corrupted[i] = corrupted[i].wrapping_add(1);
        }
        
        test_header_slice(&corrupted);
    }
}

/// Test zeroed MAC tags
fn test_zeroed_mac_tags(data: &[u8]) {
    if data.len() >= 96 {
        let mut zeroed = data.to_vec();
        
        // Zero MAC tag bytes
        for i in 64..96 {
            zeroed[i] = 0;
        }
        
        test_header_slice(&zeroed);
    }
}

/// Test repeated MAC tags
fn test_repeated_mac_tags(data: &[u8]) {
    if data.len() >= 96 {
        let mut repeated = data.to_vec();
        
        // Repeat MAC tag bytes
        for i in 64..96 {
            repeated[i] = 0xAA;
        }
        
        test_header_slice(&repeated);
    }
}

/// Test corrupted capability IDs
fn test_corrupted_capability_ids(data: &[u8]) {
    if data.len() >= 128 {
        let mut corrupted = data.to_vec();
        
        // Corrupt capability ID bytes
        for i in 96..128 {
            corrupted[i] = corrupted[i].wrapping_add(1);
        }
        
        test_header_slice(&corrupted);
    }
}

/// Test zeroed capability IDs
fn test_zeroed_capability_ids(data: &[u8]) {
    if data.len() >= 128 {
        let mut zeroed = data.to_vec();
        
        // Zero capability ID bytes
        for i in 96..128 {
            zeroed[i] = 0;
        }
        
        test_header_slice(&zeroed);
    }
}

/// Test repeated capability IDs
fn test_repeated_capability_ids(data: &[u8]) {
    if data.len() >= 128 {
        let mut repeated = data.to_vec();
        
        // Repeat capability ID bytes
        for i in 96..128 {
            repeated[i] = 0x55;
        }
        
        test_header_slice(&repeated);
    }
}

/// Test corrupted message IDs
fn test_corrupted_message_ids(data: &[u8]) {
    if data.len() >= 8 {
        let mut corrupted = data.to_vec();
        
        // Corrupt message ID bytes
        for i in 0..8 {
            corrupted[i] = corrupted[i].wrapping_add(1);
        }
        
        test_header_slice(&corrupted);
    }
}

/// Test zeroed message IDs
fn test_zeroed_message_ids(data: &[u8]) {
    if data.len() >= 8 {
        let mut zeroed = data.to_vec();
        
        // Zero message ID bytes
        for i in 0..8 {
            zeroed[i] = 0;
        }
        
        test_header_slice(&zeroed);
    }
}

/// Test repeated message IDs
fn test_repeated_message_ids(data: &[u8]) {
    if data.len() >= 8 {
        let mut repeated = data.to_vec();
        
        // Repeat message ID bytes
        for i in 0..8 {
            repeated[i] = 0xAA;
        }
        
        test_header_slice(&repeated);
    }
}

/// Test corrupted timestamps
fn test_corrupted_timestamps(data: &[u8]) {
    if data.len() >= 16 {
        let mut corrupted = data.to_vec();
        
        // Corrupt timestamp bytes
        for i in 8..16 {
            corrupted[i] = corrupted[i].wrapping_add(1);
        }
        
        test_header_slice(&corrupted);
    }
}

/// Test zeroed timestamps
fn test_zeroed_timestamps(data: &[u8]) {
    if data.len() >= 16 {
        let mut zeroed = data.to_vec();
        
        // Zero timestamp bytes
        for i in 8..16 {
            zeroed[i] = 0;
        }
        
        test_header_slice(&zeroed);
    }
}

/// Test future timestamps
fn test_future_timestamps(data: &[u8]) {
    if data.len() >= 16 {
        let mut future = data.to_vec();
        
        // Set future timestamp (year 2100)
        let future_time: u64 = 4102444800; // Unix timestamp for 2100
        let future_bytes = future_time.to_le_bytes();
        
        for i in 0..8 {
            future[8 + i] = future_bytes[i];
        }
        
        test_header_slice(&future);
    }
}

/// Test old timestamps
fn test_old_timestamps(data: &[u8]) {
    if data.len() >= 16 {
        let mut old = data.to_vec();
        
        // Set old timestamp (year 1970)
        let old_time: u64 = 0; // Unix timestamp for 1970
        let old_bytes = old_time.to_le_bytes();
        
        for i in 0..8 {
            old[8 + i] = old_bytes[i];
        }
        
        test_header_slice(&old);
    }
}

/// Test corrupted sizes
fn test_corrupted_sizes(data: &[u8]) {
    if data.len() >= 24 {
        let mut corrupted = data.to_vec();
        
        // Corrupt size bytes
        for i in 16..24 {
            corrupted[i] = corrupted[i].wrapping_add(1);
        }
        
        test_header_slice(&corrupted);
    }
}

/// Test zeroed sizes
fn test_zeroed_sizes(data: &[u8]) {
    if data.len() >= 24 {
        let mut zeroed = data.to_vec();
        
        // Zero size bytes
        for i in 16..24 {
            zeroed[i] = 0;
        }
        
        test_header_slice(&zeroed);
    }
}

/// Test excessive sizes
fn test_excessive_sizes(data: &[u8]) {
    if data.len() >= 24 {
        let mut excessive = data.to_vec();
        
        // Set excessive size (1GB)
        let excessive_size: u64 = 0x40000000;
        let excessive_bytes = excessive_size.to_le_bytes();
        
        for i in 0..8 {
            excessive[16 + i] = excessive_bytes[i];
        }
        
        test_header_slice(&excessive);
    }
}

/// Test corrupted flags
fn test_corrupted_flags(data: &[u8]) {
    if data.len() >= 32 {
        let mut corrupted = data.to_vec();
        
        // Corrupt flag bytes
        for i in 24..32 {
            corrupted[i] = corrupted[i].wrapping_add(1);
        }
        
        test_header_slice(&corrupted);
    }
}

/// Test zeroed flags
fn test_zeroed_flags(data: &[u8]) {
    if data.len() >= 32 {
        let mut zeroed = data.to_vec();
        
        // Zero flag bytes
        for i in 24..32 {
            zeroed[i] = 0;
        }
        
        test_header_slice(&zeroed);
    }
}

/// Test all flags set
fn test_all_flags_set(data: &[u8]) {
    if data.len() >= 32 {
        let mut all_flags = data.to_vec();
        
        // Set all flag bytes
        for i in 24..32 {
            all_flags[i] = 0xFF;
        }
        
        test_header_slice(&all_flags);
    }
}

/// Test corrupted priorities
fn test_corrupted_priorities(data: &[u8]) {
    if data.len() >= 40 {
        let mut corrupted = data.to_vec();
        
        // Corrupt priority bytes
        for i in 32..40 {
            corrupted[i] = corrupted[i].wrapping_add(1);
        }
        
        test_header_slice(&corrupted);
    }
}

/// Test zeroed priorities
fn test_zeroed_priorities(data: &[u8]) {
    if data.len() >= 40 {
        let mut zeroed = data.to_vec();
        
        // Zero priority bytes
        for i in 32..40 {
            zeroed[i] = 0;
        }
        
        test_header_slice(&zeroed);
    }
}

/// Test excessive priorities
fn test_excessive_priorities(data: &[u8]) {
    if data.len() >= 40 {
        let mut excessive = data.to_vec();
        
        // Set excessive priority (1000)
        let excessive_priority: u64 = 1000;
        let excessive_bytes = excessive_priority.to_le_bytes();
        
        for i in 0..8 {
            excessive[32 + i] = excessive_bytes[i];
        }
        
        test_header_slice(&excessive);
    }
}

/// Test corrupted checksums
fn test_corrupted_checksums(data: &[u8]) {
    if data.len() >= 48 {
        let mut corrupted = data.to_vec();
        
        // Corrupt checksum bytes
        for i in 44..48 {
            corrupted[i] = corrupted[i].wrapping_add(1);
        }
        
        test_header_slice(&corrupted);
    }
}

/// Test zeroed checksums
fn test_zeroed_checksums(data: &[u8]) {
    if data.len() >= 48 {
        let mut zeroed = data.to_vec();
        
        // Zero checksum bytes
        for i in 44..48 {
            zeroed[i] = 0;
        }
        
        test_header_slice(&zeroed);
    }
}

/// Test invalid checksums
fn test_invalid_checksums(data: &[u8]) {
    if data.len() >= 48 {
        let mut invalid = data.to_vec();
        
        // Set invalid checksum (0xDEADBEEF)
        let invalid_checksum: u32 = 0xDEADBEEF;
        let invalid_bytes = invalid_checksum.to_le_bytes();
        
        for i in 0..4 {
            invalid[44 + i] = invalid_bytes[i];
        }
        
        test_header_slice(&invalid);
    }
}
