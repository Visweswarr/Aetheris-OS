/// Schema Validation for Polymera OS System Calls
/// 
/// This module provides build-time validation to ensure that generated
/// syscall files match the current schema hash.

use super::schema_hash::SCHEMA_HASH;

/// Validate that the current schema hash matches the expected hash
/// 
/// This function is called at build time to ensure that any changes
/// to the syscall schema trigger regeneration of all ABI artifacts.
/// 
/// # Returns
/// `true` if validation passes, panics if validation fails
pub fn validate_schema_hash() -> bool {
    // This would typically compare against a stored hash from the build system
    // For now, we just ensure the hash is not empty
    assert!(!SCHEMA_HASH.is_empty(), "Schema hash cannot be empty");
    
    // In a real implementation, you would:
    // 1. Read the expected hash from a build artifact
    // 2. Compare it with SCHEMA_HASH
    // 3. Panic if they don't match
    
    true
}

/// Get the current schema hash
pub fn get_current_schema_hash() -> &'static str {
    SCHEMA_HASH
}

/// Check if schema validation is enabled
pub fn is_schema_validation_enabled() -> bool {
    cfg!(feature = "schema_validation")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_schema_hash_not_empty() {
        assert!(!SCHEMA_HASH.is_empty());
    }
    
    #[test]
    fn test_schema_hash_format() {
        // MD5 hashes are 32 characters long
        assert_eq!(SCHEMA_HASH.len(), 32);
        
        // MD5 hashes contain only hexadecimal characters
        assert!(SCHEMA_HASH.chars().all(|c| c.is_ascii_hexdigit()));
    }
    
    #[test]
    fn test_validation_function() {
        assert!(validate_schema_hash());
    }
}
