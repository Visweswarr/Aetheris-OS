//! ELF Header Parser
//! 
//! This module provides a minimal ELF64 header parser for validating
//! ELF file headers. It validates magic numbers, class, and endianness
//! but does not yet map segments.

use core::mem::size_of;
use alloc::vec::Vec;

/// ELF magic number constants
pub const ELF_MAGIC: [u8; 4] = [0x7f, 0x45, 0x4c, 0x46]; // "\x7fELF"

/// ELF class constants
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfClass {
    None = 0,
    Elf32 = 1,
    Elf64 = 2,
}

/// ELF data encoding (endianness) constants
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfData {
    None = 0,
    LittleEndian = 1,
    BigEndian = 2,
}

/// ELF version constants
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfVersion {
    None = 0,
    Current = 1,
}

/// ELF OS/ABI constants
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfOsAbi {
    None = 0,
    SystemV = 0,
    HPUX = 1,
    NetBSD = 2,
    Linux = 3,
    Solaris = 6,
    AIX = 7,
    IRIX = 8,
    FreeBSD = 9,
    OpenBSD = 12,
}

/// ELF file type constants
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfType {
    None = 0,
    Relocatable = 1,
    Executable = 2,
    Shared = 3,
    Core = 4,
}

/// ELF machine architecture constants
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfMachine {
    None = 0,
    X86 = 3,
    X86_64 = 62,
    AArch64 = 183,
    RiscV = 243,
}

/// ELF64 header structure
/// 
/// This represents the ELF64 header as defined in the ELF specification.
/// All fields are stored in the target endianness.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Elf64Header {
    /// ELF identification
    pub e_ident: [u8; 16],
    
    /// Object file type
    pub e_type: u16,
    
    /// Machine architecture
    pub e_machine: u16,
    
    /// Object file version
    pub e_version: u32,
    
    /// Entry point virtual address
    pub e_entry: u64,
    
    /// Program header table file offset
    pub e_phoff: u64,
    
    /// Section header table file offset
    pub e_shoff: u64,
    
    /// Processor-specific flags
    pub e_flags: u32,
    
    /// ELF header size in bytes
    pub e_ehsize: u16,
    
    /// Program header table entry size
    pub e_phentsize: u16,
    
    /// Program header table entry count
    pub e_phnum: u16,
    
    /// Section header table entry size
    pub e_shentsize: u16,
    
    /// Section header table entry count
    pub e_shnum: u16,
    
    /// Section header string table index
    pub e_shstrndx: u16,
}

impl Elf64Header {
    /// Create a new ELF64 header with default values
    pub fn new() -> Self {
        Self {
            e_ident: [0; 16],
            e_type: 0,
            e_machine: 0,
            e_version: 0,
            e_entry: 0,
            e_phoff: 0,
            e_shoff: 0,
            e_flags: 0,
            e_ehsize: size_of::<Elf64Header>() as u16,
            e_phentsize: 0,
            e_phnum: 0,
            e_shentsize: 0,
            e_shnum: 0,
            e_shstrndx: 0,
        }
    }
    
    /// Parse ELF64 header from a byte slice
    /// 
    /// # Arguments
    /// * `data` - Raw bytes containing the ELF header
    /// 
    /// # Returns
    /// * `Ok(Elf64Header)` - Successfully parsed header
    /// * `Err(ElfError)` - Parsing error
    pub fn parse(data: &[u8]) -> Result<Self, ElfError> {
        // Check minimum size
        if data.len() < size_of::<Elf64Header>() {
            return Err(ElfError::InvalidSize);
        }
        
        // Create header structure
        let mut header = Self::new();
        
        // Parse e_ident field (first 16 bytes)
        header.e_ident.copy_from_slice(&data[0..16]);
        
        // Validate magic number
        if header.e_ident[0..4] != ELF_MAGIC {
            return Err(ElfError::InvalidMagic);
        }
        
        // Validate class (must be ELF64)
        if header.e_ident[4] != ElfClass::Elf64 as u8 {
            return Err(ElfError::InvalidClass);
        }
        
        // Validate data encoding (endianness)
        match header.e_ident[5] {
            1 => header.e_ident[5] = ElfData::LittleEndian as u8,
            2 => header.e_ident[5] = ElfData::BigEndian as u8,
            _ => return Err(ElfError::InvalidDataEncoding),
        }
        
        // Validate version
        if header.e_ident[6] != ElfVersion::Current as u8 {
            return Err(ElfError::InvalidVersion);
        }
        
        // Parse remaining fields (assuming little-endian for now)
        // In a full implementation, we'd handle endianness conversion
        let offset = 16;
        
        header.e_type = u16::from_le_bytes([data[offset], data[offset + 1]]);
        header.e_machine = u16::from_le_bytes([data[offset + 2], data[offset + 3]]);
        header.e_version = u32::from_le_bytes([
            data[offset + 4], data[offset + 5], 
            data[offset + 6], data[offset + 7]
        ]);
        header.e_entry = u64::from_le_bytes([
            data[offset + 8], data[offset + 9], data[offset + 10], data[offset + 11],
            data[offset + 12], data[offset + 13], data[offset + 14], data[offset + 15]
        ]);
        header.e_phoff = u64::from_le_bytes([
            data[offset + 16], data[offset + 17], data[offset + 18], data[offset + 19],
            data[offset + 20], data[offset + 21], data[offset + 22], data[offset + 23]
        ]);
        header.e_shoff = u64::from_le_bytes([
            data[offset + 24], data[offset + 25], data[offset + 26], data[offset + 27],
            data[offset + 28], data[offset + 29], data[offset + 30], data[offset + 31]
        ]);
        header.e_flags = u32::from_le_bytes([
            data[offset + 32], data[offset + 33], 
            data[offset + 34], data[offset + 35]
        ]);
        header.e_ehsize = u16::from_le_bytes([data[offset + 36], data[offset + 37]]);
        header.e_phentsize = u16::from_le_bytes([data[offset + 38], data[offset + 39]]);
        header.e_phnum = u16::from_le_bytes([data[offset + 40], data[offset + 41]]);
        header.e_shentsize = u16::from_le_bytes([data[offset + 42], data[offset + 43]]);
        header.e_shnum = u16::from_le_bytes([data[offset + 44], data[offset + 45]]);
        header.e_shstrndx = u16::from_le_bytes([data[offset + 46], data[offset + 47]]);
        
        Ok(header)
    }
    
    /// Get the ELF class from the header
    pub fn get_class(&self) -> Result<ElfClass, ElfError> {
        match self.e_ident[4] {
            0 => Ok(ElfClass::None),
            1 => Ok(ElfClass::Elf32),
            2 => Ok(ElfClass::Elf64),
            _ => Err(ElfError::InvalidClass),
        }
    }
    
    /// Get the data encoding (endianness) from the header
    pub fn get_data_encoding(&self) -> Result<ElfData, ElfError> {
        match self.e_ident[5] {
            0 => Ok(ElfData::None),
            1 => Ok(ElfData::LittleEndian),
            2 => Ok(ElfData::BigEndian),
            _ => Err(ElfError::InvalidDataEncoding),
        }
    }
    
    /// Get the ELF version from the header
    pub fn get_version(&self) -> Result<ElfVersion, ElfError> {
        match self.e_ident[6] {
            0 => Ok(ElfVersion::None),
            1 => Ok(ElfVersion::Current),
            _ => Err(ElfError::InvalidVersion),
        }
    }
    
    /// Get the OS/ABI from the header
    pub fn get_os_abi(&self) -> ElfOsAbi {
        match self.e_ident[7] {
            0 => ElfOsAbi::None,
            1 => ElfOsAbi::HPUX,
            2 => ElfOsAbi::NetBSD,
            3 => ElfOsAbi::Linux,
            6 => ElfOsAbi::Solaris,
            7 => ElfOsAbi::AIX,
            8 => ElfOsAbi::IRIX,
            9 => ElfOsAbi::FreeBSD,
            12 => ElfOsAbi::OpenBSD,
            _ => ElfOsAbi::None,
        }
    }
    
    /// Get the file type from the header
    pub fn get_file_type(&self) -> Result<ElfType, ElfError> {
        match self.e_type {
            0 => Ok(ElfType::None),
            1 => Ok(ElfType::Relocatable),
            2 => Ok(ElfType::Executable),
            3 => Ok(ElfType::Shared),
            4 => Ok(ElfType::Core),
            _ => Err(ElfError::InvalidFileType),
        }
    }
    
    /// Get the machine architecture from the header
    pub fn get_machine(&self) -> Result<ElfMachine, ElfError> {
        match self.e_machine {
            0 => Ok(ElfMachine::None),
            3 => Ok(ElfMachine::X86),
            62 => Ok(ElfMachine::X86_64),
            183 => Ok(ElfMachine::AArch64),
            243 => Ok(ElfMachine::RiscV),
            _ => Err(ElfError::InvalidMachine),
        }
    }
    
    /// Check if this is a valid ELF64 executable
    pub fn is_valid_executable(&self) -> Result<bool, ElfError> {
        // Check class
        if self.get_class()? != ElfClass::Elf64 {
            return Ok(false);
        }
        
        // Check file type
        if self.get_file_type()? != ElfType::Executable {
            return Ok(false);
        }
        
        // Check version
        if self.get_version()? != ElfVersion::Current {
            return Ok(false);
        }
        
        // Check header size
        if self.e_ehsize != size_of::<Elf64Header>() as u16 {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    /// Get a human-readable description of the ELF file
    pub fn get_description(&self) -> Result<String, ElfError> {
        let class = self.get_class()?;
        let data = self.get_data_encoding()?;
        let version = self.get_version()?;
        let file_type = self.get_file_type()?;
        let machine = self.get_machine()?;
        let os_abi = self.get_os_abi();
        
        Ok(format!(
            "ELF64 {} {} {} {} {} {}",
            match class {
                ElfClass::Elf64 => "64-bit",
                _ => "unknown-class",
            },
            match data {
                ElfData::LittleEndian => "little-endian",
                ElfData::BigEndian => "big-endian",
                _ => "unknown-endianness",
            },
            match version {
                ElfVersion::Current => "v1",
                _ => "unknown-version",
            },
            match file_type {
                ElfType::Executable => "executable",
                ElfType::Shared => "shared-object",
                ElfType::Relocatable => "relocatable",
                ElfType::Core => "core-dump",
                _ => "unknown-type",
            },
            match machine {
                ElfMachine::X86_64 => "x86-64",
                ElfMachine::AArch64 => "aarch64",
                ElfMachine::X86 => "x86",
                ElfMachine::RiscV => "riscv",
                _ => "unknown-machine",
            },
            match os_abi {
                ElfOsAbi::Linux => "Linux",
                ElfOsAbi::FreeBSD => "FreeBSD",
                ElfOsAbi::NetBSD => "NetBSD",
                ElfOsAbi::OpenBSD => "OpenBSD",
                ElfOsAbi::Solaris => "Solaris",
                ElfOsAbi::AIX => "AIX",
                ElfOsAbi::HPUX => "HP-UX",
                ElfOsAbi::IRIX => "IRIX",
                _ => "SystemV",
            }
        ))
    }
}

/// ELF parsing errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElfError {
    /// Invalid file size
    InvalidSize,
    
    /// Invalid magic number
    InvalidMagic,
    
    /// Invalid ELF class
    InvalidClass,
    
    /// Invalid data encoding (endianness)
    InvalidDataEncoding,
    
    /// Invalid ELF version
    InvalidVersion,
    
    /// Invalid file type
    InvalidFileType,
    
    /// Invalid machine architecture
    InvalidMachine,
    
    /// Unsupported endianness
    UnsupportedEndianness,
}

impl core::fmt::Display for ElfError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ElfError::InvalidSize => write!(f, "Invalid ELF file size"),
            ElfError::InvalidMagic => write!(f, "Invalid ELF magic number"),
            ElfError::InvalidClass => write!(f, "Invalid ELF class"),
            ElfError::InvalidDataEncoding => write!(f, "Invalid data encoding"),
            ElfError::InvalidVersion => write!(f, "Invalid ELF version"),
            ElfError::InvalidFileType => write!(f, "Invalid file type"),
            ElfError::InvalidMachine => write!(f, "Invalid machine architecture"),
            ElfError::UnsupportedEndianness => write!(f, "Unsupported endianness"),
        }
    }
}

/// Create a minimal valid ELF64 header for testing
pub fn create_test_elf64_header() -> Vec<u8> {
    let mut data = Vec::new();
    
    // e_ident[0..4]: Magic number
    data.extend_from_slice(&ELF_MAGIC);
    
    // e_ident[4]: Class (ELF64)
    data.push(ElfClass::Elf64 as u8);
    
    // e_ident[5]: Data encoding (little-endian)
    data.push(ElfData::LittleEndian as u8);
    
    // e_ident[6]: Version (current)
    data.push(ElfVersion::Current as u8);
    
    // e_ident[7]: OS/ABI (Linux)
    data.push(ElfOsAbi::Linux as u8);
    
    // e_ident[8..16]: ABI version and padding
    data.extend_from_slice(&[0; 8]);
    
    // e_type: Executable
    data.extend_from_slice(&(ElfType::Executable as u16).to_le_bytes());
    
    // e_machine: x86-64
    data.extend_from_slice(&(ElfMachine::X86_64 as u16).to_le_bytes());
    
    // e_version: Current
    data.extend_from_slice(&(ElfVersion::Current as u32).to_le_bytes());
    
    // e_entry: Entry point (0x400000)
    data.extend_from_slice(&0x400000u64.to_le_bytes());
    
    // e_phoff: Program header offset (64)
    data.extend_from_slice(&64u64.to_le_bytes());
    
    // e_shoff: Section header offset (0 for now)
    data.extend_from_slice(&0u64.to_le_bytes());
    
    // e_flags: Flags (0)
    data.extend_from_slice(&0u32.to_le_bytes());
    
    // e_ehsize: Header size (64)
    data.extend_from_slice(&64u16.to_le_bytes());
    
    // e_phentsize: Program header entry size (56)
    data.extend_from_slice(&56u16.to_le_bytes());
    
    // e_phnum: Program header count (1)
    data.extend_from_slice(&1u16.to_le_bytes());
    
    // e_shentsize: Section header entry size (64)
    data.extend_from_slice(&64u16.to_le_bytes());
    
    // e_shnum: Section header count (0 for now)
    data.extend_from_slice(&0u16.to_le_bytes());
    
    // e_shstrndx: Section header string table index (0)
    data.extend_from_slice(&0u16.to_le_bytes());
    
    data
}

/// Create an invalid ELF header for testing
pub fn create_invalid_elf_header() -> Vec<u8> {
    let mut data = Vec::new();
    
    // Invalid magic number
    data.extend_from_slice(b"INVALID");
    
    // Fill rest with zeros
    data.extend_from_slice(&vec![0; size_of::<Elf64Header>() - 8]);
    
    data
}

/// Create an ELF32 header for testing (should fail)
pub fn create_elf32_header() -> Vec<u8> {
    let mut data = Vec::new();
    
    // e_ident[0..4]: Magic number
    data.extend_from_slice(&ELF_MAGIC);
    
    // e_ident[4]: Class (ELF32)
    data.push(ElfClass::Elf32 as u8);
    
    // e_ident[5]: Data encoding (little-endian)
    data.push(ElfData::LittleEndian as u8);
    
    // e_ident[6]: Version (current)
    data.push(ElfVersion::Current as u8);
    
    // e_ident[7]: OS/ABI (Linux)
    data.push(ElfOsAbi::Linux as u8);
    
    // e_ident[8..16]: ABI version and padding
    data.extend_from_slice(&[0; 8]);
    
    // Fill rest with zeros (ELF32 header is smaller)
    data.extend_from_slice(&vec![0; 32]);
    
    data
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_elf64_header_creation() {
        let header = Elf64Header::new();
        assert_eq!(header.e_ehsize, size_of::<Elf64Header>() as u16);
        assert_eq!(header.e_ident.len(), 16);
    }
    
    #[test]
    fn test_valid_elf64_header_parsing() {
        let data = create_test_elf64_header();
        let header = Elf64Header::parse(&data).unwrap();
        
        assert_eq!(header.get_class().unwrap(), ElfClass::Elf64);
        assert_eq!(header.get_data_encoding().unwrap(), ElfData::LittleEndian);
        assert_eq!(header.get_version().unwrap(), ElfVersion::Current);
        assert_eq!(header.get_file_type().unwrap(), ElfType::Executable);
        assert_eq!(header.get_machine().unwrap(), ElfMachine::X86_64);
        assert_eq!(header.get_os_abi(), ElfOsAbi::Linux);
        assert_eq!(header.e_entry, 0x400000);
        assert_eq!(header.e_phoff, 64);
        assert_eq!(header.e_ehsize, 64);
    }
    
    #[test]
    fn test_invalid_magic_number() {
        let data = create_invalid_elf_header();
        let result = Elf64Header::parse(&data);
        assert!(matches!(result, Err(ElfError::InvalidMagic)));
    }
    
    #[test]
    fn test_invalid_class() {
        let data = create_elf32_header();
        let result = Elf64Header::parse(&data);
        assert!(matches!(result, Err(ElfError::InvalidClass)));
    }
    
    #[test]
    fn test_invalid_size() {
        let data = vec![0; 32]; // Too small
        let result = Elf64Header::parse(&data);
        assert!(matches!(result, Err(ElfError::InvalidSize)));
    }
    
    #[test]
    fn test_valid_executable_check() {
        let data = create_test_elf64_header();
        let header = Elf64Header::parse(&data).unwrap();
        assert!(header.is_valid_executable().unwrap());
    }
    
    #[test]
    fn test_header_description() {
        let data = create_test_elf64_header();
        let header = Elf64Header::parse(&data).unwrap();
        let description = header.get_description().unwrap();
        
        assert!(description.contains("ELF64"));
        assert!(description.contains("64-bit"));
        assert!(description.contains("little-endian"));
        assert!(description.contains("executable"));
        assert!(description.contains("x86-64"));
        assert!(description.contains("Linux"));
    }
    
    #[test]
    fn test_enum_values() {
        assert_eq!(ElfClass::Elf64 as u8, 2);
        assert_eq!(ElfData::LittleEndian as u8, 1);
        assert_eq!(ElfVersion::Current as u8, 1);
        assert_eq!(ElfType::Executable as u16, 2);
        assert_eq!(ElfMachine::X86_64 as u16, 62);
        assert_eq!(ElfOsAbi::Linux as u8, 3);
    }
}


