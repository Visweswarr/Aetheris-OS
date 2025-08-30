/// User Task Image Header for Polymera OS
/// 
/// This module defines a minimal image header structure for user tasks
/// that will be compatible with WASI and ELF formats in the future.
/// The header includes magic validation, entry point, capability requirements,
/// stack size, and data region integrity hashes.

use crate::{kprintln, klog, kprintln};
use crate::log::Level;
use crate::secman::cap_v2::{CapTokenV2, CapValidationResult};
use core::mem;
use alloc::vec::Vec;
use alloc::string::String;

//=============================================================================
// HEADER CONSTANTS AND MAGIC VALUES
//=============================================================================

/// Magic number for Polymera OS user task images
pub const POLYMERA_MAGIC: [u8; 8] = [0x50, 0x4F, 0x4C, 0x59, 0x4D, 0x45, 0x52, 0x41]; // "POLYMERA"

/// Magic number for WASI-compatible images
pub const WASI_MAGIC: [u8; 8] = [0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00]; // WASM magic

/// Magic number for ELF-compatible images
pub const ELF_MAGIC: [u8; 4] = [0x7F, 0x45, 0x4C, 0x46]; // "\x7FELF"

/// Current header version
pub const HEADER_VERSION: u32 = 1;

/// Maximum number of required capabilities
pub const MAX_REQUIRED_CAPS: usize = 16;

/// Maximum number of data regions
pub const MAX_DATA_REGIONS: usize = 8;

/// Default stack size (64KB)
pub const DEFAULT_STACK_SIZE: usize = 64 * 1024;

/// Maximum stack size (1MB)
pub const MAX_STACK_SIZE: usize = 1024 * 1024;

//=============================================================================
// HEADER STRUCTURES
//=============================================================================

/// User task image header
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserTaskHeader {
    /// Magic number identifying the image format
    pub magic: [u8; 8],
    
    /// Header version
    pub version: u32,
    
    /// Image format type
    pub format: ImageFormat,
    
    /// Entry point address
    pub entry_point: u64,
    
    /// Required capabilities for execution
    pub required_caps: Vec<CapabilityRequirement>,
    
    /// Stack size in bytes
    pub stack_size: usize,
    
    /// Data regions with integrity hashes
    pub data_regions: Vec<DataRegion>,
    
    /// Image integrity hash
    pub image_hash: [u8; 32],
    
    /// Header checksum
    pub header_checksum: u32,
    
    /// Reserved for future use
    pub reserved: [u8; 64],
}

/// Image format types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// Polymera OS native format
    Polymera,
    /// WebAssembly System Interface
    Wasi,
    /// Executable and Linkable Format
    Elf,
    /// Unknown or unsupported format
    Unknown,
}

impl From<[u8; 8]> for ImageFormat {
    fn from(magic: [u8; 8]) -> Self {
        if magic == POLYMERA_MAGIC {
            ImageFormat::Polymera
        } else if magic == WASI_MAGIC {
            ImageFormat::Wasi
        } else if magic[0..4] == ELF_MAGIC {
            ImageFormat::Elf
        } else {
            ImageFormat::Unknown
        }
    }
}

impl core::fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ImageFormat::Polymera => write!(f, "Polymera"),
            ImageFormat::Wasi => write!(f, "WASI"),
            ImageFormat::Elf => write!(f, "ELF"),
            ImageFormat::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Capability requirement for user task execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityRequirement {
    /// Capability type/scope
    pub cap_type: String,
    
    /// Required permission level
    pub permission_level: u32,
    
    /// Resource identifier
    pub resource_id: u64,
    
    /// Capability token for validation
    pub cap_token: Option<CapTokenV2>,
}

/// Data region with integrity hash
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataRegion {
    /// Region name/identifier
    pub name: String,
    
    /// Start address
    pub start_address: u64,
    
    /// Size in bytes
    pub size: usize,
    
    /// Integrity hash (SHA-256)
    pub integrity_hash: [u8; 32],
    
    /// Memory protection flags
    pub protection: MemoryProtection,
}

/// Memory protection flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryProtection {
    /// Read permission
    pub read: bool,
    /// Write permission
    pub write: bool,
    /// Execute permission
    pub execute: bool,
    /// Shared memory
    pub shared: bool,
}

impl Default for MemoryProtection {
    fn default() -> Self {
        Self {
            read: true,
            write: false,
            execute: false,
            shared: false,
        }
    }
}

impl core::fmt::Display for MemoryProtection {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut perms = String::new();
        if self.read { perms.push('r'); } else { perms.push('-'); }
        if self.write { perms.push('w'); } else { perms.push('-'); }
        if self.execute { perms.push('x'); } else { perms.push('-'); }
        if self.shared { perms.push('s'); } else { perms.push('-'); }
        write!(f, "{}", perms)
    }
}

/// Header parsing result
#[derive(Debug, Clone)]
pub struct HeaderParseResult {
    /// Whether parsing was successful
    pub success: bool,
    
    /// Parsed header if successful
    pub header: Option<UserTaskHeader>,
    
    /// Error message if parsing failed
    pub error: Option<String>,
    
    /// Validation warnings
    pub warnings: Vec<String>,
}

//=============================================================================
// HEADER PARSER IMPLEMENTATION
//=============================================================================

impl UserTaskHeader {
    /// Create a new user task header
    pub fn new(
        format: ImageFormat,
        entry_point: u64,
        stack_size: usize,
    ) -> Self {
        let magic = match format {
            ImageFormat::Polymera => POLYMERA_MAGIC,
            ImageFormat::Wasi => WASI_MAGIC,
            ImageFormat::Elf => [ELF_MAGIC[0], ELF_MAGIC[1], ELF_MAGIC[2], ELF_MAGIC[3], 0, 0, 0, 0],
            ImageFormat::Unknown => [0; 8],
        };
        
        Self {
            magic,
            version: HEADER_VERSION,
            format,
            entry_point,
            required_caps: Vec::new(),
            stack_size: stack_size.min(MAX_STACK_SIZE),
            data_regions: Vec::new(),
            image_hash: [0; 32],
            header_checksum: 0,
            reserved: [0; 64],
        }
    }
    
    /// Parse header from binary data
    /// 
    /// # Arguments
    /// * `data` - Binary data containing the header
    /// 
    /// # Returns
    /// `HeaderParseResult` with parsing result
    pub fn parse(data: &[u8]) -> HeaderParseResult {
        if data.len() < mem::size_of::<UserTaskHeader>() {
            return HeaderParseResult {
                success: false,
                header: None,
                error: Some("Data too short for header".to_string()),
                warnings: Vec::new(),
            };
        }
        
        let mut warnings = Vec::new();
        
        // Extract magic number
        let mut magic = [0u8; 8];
        magic.copy_from_slice(&data[0..8]);
        
        // Determine format from magic
        let format = ImageFormat::from(magic);
        if format == ImageFormat::Unknown {
            return HeaderParseResult {
                success: false,
                header: None,
                error: Some("Unknown or unsupported image format".to_string()),
                warnings: Vec::new(),
            };
        }
        
        // Extract version
        let version = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        if version != HEADER_VERSION {
            warnings.push(format!("Header version {} differs from expected {}", version, HEADER_VERSION));
        }
        
        // Extract entry point
        let entry_point = u64::from_le_bytes([
            data[12], data[13], data[14], data[15],
            data[16], data[17], data[18], data[19]
        ]);
        
        // Extract stack size
        let stack_size = usize::from_le_bytes([
            data[20], data[21], data[22], data[23],
            data[24], data[25], data[26], data[27]
        ]);
        
        // Validate stack size
        if stack_size > MAX_STACK_SIZE {
            warnings.push(format!("Stack size {} exceeds maximum {}", stack_size, MAX_STACK_SIZE));
        }
        
        // Extract required capabilities count
        let caps_count = u32::from_le_bytes([data[28], data[29], data[30], data[31]]) as usize;
        if caps_count > MAX_REQUIRED_CAPS {
            return HeaderParseResult {
                success: false,
                header: None,
                error: Some(format!("Too many required capabilities: {}", caps_count)),
                warnings,
            };
        }
        
        // Extract data regions count
        let regions_count = u32::from_le_bytes([data[32], data[33], data[34], data[35]]) as usize;
        if regions_count > MAX_DATA_REGIONS {
            return HeaderParseResult {
                success: false,
                header: None,
                error: Some(format!("Too many data regions: {}", regions_count)),
                warnings,
            };
        }
        
        // Extract image hash
        let mut image_hash = [0u8; 32];
        image_hash.copy_from_slice(&data[36..68]);
        
        // Extract header checksum
        let header_checksum = u32::from_le_bytes([data[68], data[69], data[70], data[71]]);
        
        // Extract reserved data
        let mut reserved = [0u8; 64];
        reserved.copy_from_slice(&data[72..136]);
        
        // Create header (capabilities and data regions will be empty for now)
        let header = UserTaskHeader {
            magic,
            version,
            format,
            entry_point,
            required_caps: Vec::new(), // TODO: Parse capabilities
            stack_size,
            data_regions: Vec::new(), // TODO: Parse data regions
            image_hash,
            header_checksum,
            reserved,
        };
        
        HeaderParseResult {
            success: true,
            header: Some(header),
            error: None,
            warnings,
        }
    }
    
    /// Validate header integrity
    /// 
    /// # Returns
    /// `Result<(), String>` - Success or error message
    pub fn validate(&self) -> Result<(), String> {
        // Check magic number
        if self.magic == [0; 8] {
            return Err("Invalid magic number".to_string());
        }
        
        // Check version
        if self.version != HEADER_VERSION {
            return Err(format!("Unsupported header version: {}", self.version));
        }
        
        // Check entry point
        if self.entry_point == 0 {
            return Err("Invalid entry point: 0x0".to_string());
        }
        
        // Check stack size
        if self.stack_size == 0 || self.stack_size > MAX_STACK_SIZE {
            return Err(format!("Invalid stack size: {}", self.stack_size));
        }
        
        // Check required capabilities
        if self.required_caps.len() > MAX_REQUIRED_CAPS {
            return Err(format!("Too many required capabilities: {}", self.required_caps.len()));
        }
        
        // Check data regions
        if self.data_regions.len() > MAX_DATA_REGIONS {
            return Err(format!("Too many data regions: {}", self.data_regions.len()));
        }
        
        // Validate checksum
        let calculated_checksum = self.calculate_checksum();
        if self.header_checksum != calculated_checksum {
            return Err(format!("Header checksum mismatch: expected 0x{:08x}, got 0x{:08x}", 
                             calculated_checksum, self.header_checksum));
        }
        
        Ok(())
    }
    
    /// Calculate header checksum
    /// 
    /// # Returns
    /// `u32` - Calculated checksum
    pub fn calculate_checksum(&self) -> u32 {
        // Simple checksum calculation (can be enhanced with CRC32 or similar)
        let mut checksum: u32 = 0;
        
        // Add magic bytes
        for &byte in &self.magic {
            checksum = checksum.wrapping_add(byte as u32);
        }
        
        // Add version
        checksum = checksum.wrapping_add(self.version);
        
        // Add entry point (lower 32 bits)
        checksum = checksum.wrapping_add((self.entry_point & 0xFFFFFFFF) as u32);
        
        // Add stack size
        checksum = checksum.wrapping_add(self.stack_size as u32);
        
        // Add required capabilities count
        checksum = checksum.wrapping_add(self.required_caps.len() as u32);
        
        // Add data regions count
        checksum = checksum.wrapping_add(self.data_regions.len() as u32);
        
        checksum
    }
    
    /// Serialize header to binary data
    /// 
    /// # Returns
    /// `Vec<u8>` - Serialized header data
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();
        
        // Magic number
        data.extend_from_slice(&self.magic);
        
        // Version
        data.extend_from_slice(&self.version.to_le_bytes());
        
        // Entry point
        data.extend_from_slice(&self.entry_point.to_le_bytes());
        
        // Stack size
        data.extend_from_slice(&(self.stack_size as u64).to_le_bytes());
        
        // Required capabilities count
        data.extend_from_slice(&(self.required_caps.len() as u32).to_le_bytes());
        
        // Data regions count
        data.extend_from_slice(&(self.data_regions.len() as u32).to_le_bytes());
        
        // Image hash
        data.extend_from_slice(&self.image_hash);
        
        // Header checksum
        data.extend_from_slice(&self.header_checksum.to_le_bytes());
        
        // Reserved
        data.extend_from_slice(&self.reserved);
        
        data
    }
    
    /// Add required capability
    /// 
    /// # Arguments
    /// * `cap_type` - Capability type
    /// * `permission_level` - Required permission level
    /// * `resource_id` - Resource identifier
    /// 
    /// # Returns
    /// `Result<(), String>` - Success or error
    pub fn add_required_capability(
        &mut self,
        cap_type: String,
        permission_level: u32,
        resource_id: u64,
    ) -> Result<(), String> {
        if self.required_caps.len() >= MAX_REQUIRED_CAPS {
            return Err("Maximum number of required capabilities reached".to_string());
        }
        
        let cap_req = CapabilityRequirement {
            cap_type,
            permission_level,
            resource_id,
            cap_token: None,
        };
        
        self.required_caps.push(cap_req);
        Ok(())
    }
    
    /// Add data region
    /// 
    /// # Arguments
    /// * `name` - Region name
    /// * `start_address` - Start address
    /// * `size` - Size in bytes
    /// * `integrity_hash` - Integrity hash
    /// * `protection` - Memory protection flags
    /// 
    /// # Returns
    /// `Result<(), String>` - Success or error
    pub fn add_data_region(
        &mut self,
        name: String,
        start_address: u64,
        size: usize,
        integrity_hash: [u8; 32],
        protection: MemoryProtection,
    ) -> Result<(), String> {
        if self.data_regions.len() >= MAX_DATA_REGIONS {
            return Err("Maximum number of data regions reached".to_string());
        }
        
        let region = DataRegion {
            name,
            start_address,
            size,
            integrity_hash,
            protection,
        };
        
        self.data_regions.push(region);
        Ok(())
    }
    
    /// Set image hash
    /// 
    /// # Arguments
    /// * `hash` - Image integrity hash
    pub fn set_image_hash(&mut self, hash: [u8; 32]) {
        self.image_hash = hash;
    }
    
    /// Update header checksum
    pub fn update_checksum(&mut self) {
        self.header_checksum = self.calculate_checksum();
    }
}

//=============================================================================
// TEST IMAGE CREATION
//=============================================================================

/// Create a minimal test image header for "user echo" blob
/// 
/// # Returns
/// `UserTaskHeader` - Test header
pub fn create_test_header() -> UserTaskHeader {
    let mut header = UserTaskHeader::new(
        ImageFormat::Polymera,
        0x1000, // Entry point at 0x1000
        DEFAULT_STACK_SIZE, // 64KB stack
    );
    
    // Add some test capabilities
    header.add_required_capability("SYS_EXIT".to_string(), 1, 0).unwrap();
    header.add_required_capability("SYS_WRITE".to_string(), 1, 1).unwrap();
    
    // Add test data region
    header.add_data_region(
        "text".to_string(),
        0x1000,
        0x1000,
        [0xAA; 32], // Test hash
        MemoryProtection {
            read: true,
            write: false,
            execute: true,
            shared: false,
        },
    ).unwrap();
    
    // Set test image hash
    header.set_image_hash([0xBB; 32]);
    
    // Update checksum
    header.update_checksum();
    
    header
}

/// Create an invalid test header for testing error handling
/// 
/// # Returns
/// `UserTaskHeader` - Invalid header
pub fn create_invalid_header() -> UserTaskHeader {
    let mut header = UserTaskHeader::new(
        ImageFormat::Polymera,
        0, // Invalid entry point
        0, // Invalid stack size
    );
    
    // Don't update checksum to make it invalid
    
    header
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_header_creation() {
        let header = UserTaskHeader::new(
            ImageFormat::Polymera,
            0x1000,
            0x2000,
        );
        
        assert_eq!(header.magic, POLYMERA_MAGIC);
        assert_eq!(header.version, HEADER_VERSION);
        assert_eq!(header.format, ImageFormat::Polymera);
        assert_eq!(header.entry_point, 0x1000);
        assert_eq!(header.stack_size, 0x2000);
        assert_eq!(header.required_caps.len(), 0);
        assert_eq!(header.data_regions.len(), 0);
    }
    
    #[test]
    fn test_format_detection() {
        assert_eq!(ImageFormat::from(POLYMERA_MAGIC), ImageFormat::Polymera);
        assert_eq!(ImageFormat::from(WASI_MAGIC), ImageFormat::Wasi);
        
        let elf_magic = [ELF_MAGIC[0], ELF_MAGIC[1], ELF_MAGIC[2], ELF_MAGIC[3], 0, 0, 0, 0];
        assert_eq!(ImageFormat::from(elf_magic), ImageFormat::Elf);
        
        let unknown_magic = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];
        assert_eq!(ImageFormat::from(unknown_magic), ImageFormat::Unknown);
    }
    
    #[test]
    fn test_header_validation() {
        let mut header = create_test_header();
        
        // Valid header should pass validation
        assert!(header.validate().is_ok());
        
        // Invalid entry point should fail
        header.entry_point = 0;
        assert!(header.validate().is_err());
        
        // Reset and test invalid stack size
        header = create_test_header();
        header.stack_size = 0;
        assert!(header.validate().is_err());
        
        // Reset and test invalid checksum
        header = create_test_header();
        header.header_checksum = 0x12345678;
        assert!(header.validate().is_err());
    }
    
    #[test]
    fn test_header_serialization() {
        let header = create_test_header();
        let data = header.serialize();
        
        // Data should be at least as large as the header
        assert!(data.len() >= mem::size_of::<UserTaskHeader>());
        
        // Magic number should be at the beginning
        assert_eq!(&data[0..8], &POLYMERA_MAGIC);
        
        // Version should be correct
        let version = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        assert_eq!(version, HEADER_VERSION);
    }
    
    #[test]
    fn test_header_parsing() {
        let header = create_test_header();
        let data = header.serialize();
        
        let result = UserTaskHeader::parse(&data);
        assert!(result.success);
        
        let parsed_header = result.header.unwrap();
        assert_eq!(parsed_header.magic, header.magic);
        assert_eq!(parsed_header.version, header.version);
        assert_eq!(parsed_header.format, header.format);
        assert_eq!(parsed_header.entry_point, header.entry_point);
        assert_eq!(parsed_header.stack_size, header.stack_size);
    }
    
    #[test]
    fn test_invalid_header_parsing() {
        let header = create_invalid_header();
        let data = header.serialize();
        
        let result = UserTaskHeader::parse(&data);
        assert!(result.success); // Parsing succeeds
        
        let parsed_header = result.header.unwrap();
        assert!(parsed_header.validate().is_err()); // But validation fails
    }
    
    #[test]
    fn test_capability_management() {
        let mut header = UserTaskHeader::new(ImageFormat::Polymera, 0x1000, 0x2000);
        
        // Add capabilities
        header.add_required_capability("SYS_EXIT".to_string(), 1, 0).unwrap();
        header.add_required_capability("SYS_WRITE".to_string(), 1, 1).unwrap();
        
        assert_eq!(header.required_caps.len(), 2);
        assert_eq!(header.required_caps[0].cap_type, "SYS_EXIT");
        assert_eq!(header.required_caps[1].cap_type, "SYS_WRITE");
        
        // Test maximum capabilities limit
        for i in 0..MAX_REQUIRED_CAPS {
            header.add_required_capability(format!("CAP_{}", i), 1, i as u64).unwrap();
        }
        
        // Next addition should fail
        assert!(header.add_required_capability("EXTRA_CAP".to_string(), 1, 999).is_err());
    }
    
    #[test]
    fn test_data_region_management() {
        let mut header = UserTaskHeader::new(ImageFormat::Polymera, 0x1000, 0x2000);
        
        // Add data regions
        header.add_data_region(
            "text".to_string(),
            0x1000,
            0x1000,
            [0xAA; 32],
            MemoryProtection::default(),
        ).unwrap();
        
        assert_eq!(header.data_regions.len(), 1);
        assert_eq!(header.data_regions[0].name, "text");
        assert_eq!(header.data_regions[0].start_address, 0x1000);
        assert_eq!(header.data_regions[0].size, 0x1000);
        
        // Test maximum regions limit
        for i in 0..MAX_DATA_REGIONS {
            header.add_data_region(
                format!("REGION_{}", i),
                i as u64 * 0x1000,
                0x1000,
                [0xAA; 32],
                MemoryProtection::default(),
            ).unwrap();
        }
        
        // Next addition should fail
        assert!(header.add_data_region(
            "EXTRA_REGION".to_string(),
            0x10000,
            0x1000,
            [0xFF; 32],
            MemoryProtection::default(),
        ).is_err());
    }
    
    #[test]
    fn test_memory_protection_display() {
        let protection = MemoryProtection {
            read: true,
            write: false,
            execute: true,
            shared: false,
        };
        
        assert_eq!(format!("{}", protection), "r-x-");
        
        let protection = MemoryProtection {
            read: true,
            write: true,
            execute: false,
            shared: true,
        };
        
        assert_eq!(format!("{}", protection), "rw-s");
    }
}
