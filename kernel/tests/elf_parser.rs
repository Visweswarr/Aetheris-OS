//! ELF Header Parser Tests
//! 
//! These tests verify that the ELF64 header parser correctly validates
//! magic numbers, class, endianness, and other header fields.

use crate::exec::elf::{
    Elf64Header, ElfClass, ElfData, ElfVersion, ElfOsAbi, 
    ElfType, ElfMachine, ElfError,
    create_test_elf64_header, create_invalid_elf_header, create_elf32_header
};

/// Test basic ELF64 header creation and properties
pub fn test_elf64_header_basic() -> Result<(), &'static str> {
    kprintln!("Testing basic ELF64 header functionality...");
    
    // Test header creation
    let header = Elf64Header::new();
    assert_eq!(header.e_ehsize, 64, "Header size should be 64 bytes");
    assert_eq!(header.e_ident.len(), 16, "e_ident should be 16 bytes");
    
    kprintln!("  ✅ Basic header creation working");
    
    // Test default values
    assert_eq!(header.e_type, 0, "Default e_type should be 0");
    assert_eq!(header.e_machine, 0, "Default e_machine should be 0");
    assert_eq!(header.e_version, 0, "Default e_version should be 0");
    assert_eq!(header.e_entry, 0, "Default e_entry should be 0");
    
    kprintln!("  ✅ Default values correct");
    
    kprintln!("✅ Basic ELF64 header functionality tests passed!");
    Ok(())
}

/// Test valid ELF64 header parsing
pub fn test_valid_elf64_header_parsing() -> Result<(), &'static str> {
    kprintln!("Testing valid ELF64 header parsing...");
    
    // Create and parse test header
    let test_data = create_test_elf64_header();
    let header = Elf64Header::parse(&test_data)
        .map_err(|_| "Failed to parse valid ELF64 header")?;
    
    // Test magic number validation
    assert_eq!(header.e_ident[0..4], [0x7f, 0x45, 0x4c, 0x46], "Magic number should be correct");
    kprintln!("  ✅ Magic number validation passed");
    
    // Test class validation
    assert_eq!(header.get_class()?, ElfClass::Elf64, "Class should be ELF64");
    kprintln!("  ✅ Class validation passed");
    
    // Test data encoding validation
    assert_eq!(header.get_data_encoding()?, ElfData::LittleEndian, "Data encoding should be little-endian");
    kprintln!("  ✅ Data encoding validation passed");
    
    // Test version validation
    assert_eq!(header.get_version()?, ElfVersion::Current, "Version should be current");
    kprintln!("  ✅ Version validation passed");
    
    // Test OS/ABI
    assert_eq!(header.get_os_abi(), ElfOsAbi::Linux, "OS/ABI should be Linux");
    kprintln!("  ✅ OS/ABI validation passed");
    
    // Test file type
    assert_eq!(header.get_file_type()?, ElfType::Executable, "File type should be executable");
    kprintln!("  ✅ File type validation passed");
    
    // Test machine architecture
    assert_eq!(header.get_machine()?, ElfMachine::X86_64, "Machine should be x86-64");
    kprintln!("  ✅ Machine architecture validation passed");
    
    // Test specific field values
    assert_eq!(header.e_entry, 0x400000, "Entry point should be 0x400000");
    assert_eq!(header.e_phoff, 64, "Program header offset should be 64");
    assert_eq!(header.e_ehsize, 64, "Header size should be 64");
    assert_eq!(header.e_phentsize, 56, "Program header entry size should be 56");
    assert_eq!(header.e_phnum, 1, "Program header count should be 1");
    
    kprintln!("  ✅ Field value validation passed");
    
    kprintln!("✅ Valid ELF64 header parsing tests passed!");
    Ok(())
}

/// Test invalid ELF header rejection
pub fn test_invalid_elf_header_rejection() -> Result<(), &'static str> {
    kprintln!("Testing invalid ELF header rejection...");
    
    // Test invalid magic number
    let invalid_data = create_invalid_elf_header();
    let result = Elf64Header::parse(&invalid_data);
    assert!(matches!(result, Err(ElfError::InvalidMagic)), 
            "Invalid magic number should be rejected");
    kprintln!("  ✅ Invalid magic number rejected");
    
    // Test ELF32 header (wrong class)
    let elf32_data = create_elf32_header();
    let result = Elf64Header::parse(&elf32_data);
    assert!(matches!(result, Err(ElfError::InvalidClass)), 
            "ELF32 header should be rejected");
    kprintln!("  ✅ ELF32 header rejected");
    
    // Test insufficient data size
    let small_data = vec![0; 32]; // Too small for ELF64 header
    let result = Elf64Header::parse(&small_data);
    assert!(matches!(result, Err(ElfError::InvalidSize)), 
            "Insufficient data size should be rejected");
    kprintln!("  ✅ Insufficient data size rejected");
    
    // Test empty data
    let empty_data = vec![];
    let result = Elf64Header::parse(&empty_data);
    assert!(matches!(result, Err(ElfError::InvalidSize)), 
            "Empty data should be rejected");
    kprintln!("  ✅ Empty data rejected");
    
    kprintln!("✅ Invalid ELF header rejection tests passed!");
    Ok(())
}

/// Test ELF header validation methods
pub fn test_elf_header_validation() -> Result<(), &'static str> {
    kprintln!("Testing ELF header validation methods...");
    
    // Test valid executable validation
    let test_data = create_test_elf64_header();
    let header = Elf64Header::parse(&test_data)?;
    
    let is_valid = header.is_valid_executable()?;
    assert!(is_valid, "Valid ELF64 executable should pass validation");
    kprintln!("  ✅ Valid executable validation passed");
    
    // Test header description
    let description = header.get_description()?;
    assert!(description.contains("ELF64"), "Description should contain ELF64");
    assert!(description.contains("64-bit"), "Description should contain 64-bit");
    assert!(description.contains("little-endian"), "Description should contain little-endian");
    assert!(description.contains("executable"), "Description should contain executable");
    assert!(description.contains("x86-64"), "Description should contain x86-64");
    assert!(description.contains("Linux"), "Description should contain Linux");
    kprintln!("  ✅ Header description generation passed");
    
    kprintln!("✅ ELF header validation tests passed!");
    Ok(())
}

/// Test enum value constants
pub fn test_elf_enum_constants() -> Result<(), &'static str> {
    kprintln!("Testing ELF enum constants...");
    
    // Test ELF class values
    assert_eq!(ElfClass::None as u8, 0, "ElfClass::None should be 0");
    assert_eq!(ElfClass::Elf32 as u8, 1, "ElfClass::Elf32 should be 1");
    assert_eq!(ElfClass::Elf64 as u8, 2, "ElfClass::Elf64 should be 2");
    kprintln!("  ✅ ELF class constants correct");
    
    // Test ELF data encoding values
    assert_eq!(ElfData::None as u8, 0, "ElfData::None should be 0");
    assert_eq!(ElfData::LittleEndian as u8, 1, "ElfData::LittleEndian should be 1");
    assert_eq!(ElfData::BigEndian as u8, 2, "ElfData::BigEndian should be 2");
    kprintln!("  ✅ ELF data encoding constants correct");
    
    // Test ELF version values
    assert_eq!(ElfVersion::None as u8, 0, "ElfVersion::None should be 0");
    assert_eq!(ElfVersion::Current as u8, 1, "ElfVersion::Current should be 1");
    kprintln!("  ✅ ELF version constants correct");
    
    // Test ELF file type values
    assert_eq!(ElfType::None as u16, 0, "ElfType::None should be 0");
    assert_eq!(ElfType::Relocatable as u16, 1, "ElfType::Relocatable should be 1");
    assert_eq!(ElfType::Executable as u16, 2, "ElfType::Executable should be 2");
    assert_eq!(ElfType::Shared as u16, 3, "ElfType::Shared should be 3");
    assert_eq!(ElfType::Core as u16, 4, "ElfType::Core should be 4");
    kprintln!("  ✅ ELF file type constants correct");
    
    // Test ELF machine values
    assert_eq!(ElfMachine::None as u16, 0, "ElfMachine::None should be 0");
    assert_eq!(ElfMachine::X86 as u16, 3, "ElfMachine::X86 should be 3");
    assert_eq!(ElfMachine::X86_64 as u16, 62, "ElfMachine::X86_64 should be 62");
    assert_eq!(ElfMachine::AArch64 as u16, 183, "ElfMachine::AArch64 should be 183");
    assert_eq!(ElfMachine::RiscV as u16, 243, "ElfMachine::RiscV should be 243");
    kprintln!("  ✅ ELF machine constants correct");
    
    // Test ELF OS/ABI values
    assert_eq!(ElfOsAbi::None as u8, 0, "ElfOsAbi::None should be 0");
    assert_eq!(ElfOsAbi::Linux as u8, 3, "ElfOsAbi::Linux should be 3");
    assert_eq!(ElfOsAbi::FreeBSD as u8, 9, "ElfOsAbi::FreeBSD should be 9");
    assert_eq!(ElfOsAbi::OpenBSD as u8, 12, "ElfOsAbi::OpenBSD should be 12");
    kprintln!("  ✅ ELF OS/ABI constants correct");
    
    kprintln!("✅ ELF enum constants tests passed!");
    Ok(())
}

/// Test error handling and display
pub fn test_elf_error_handling() -> Result<(), &'static str> {
    kprintln!("Testing ELF error handling...");
    
    // Test error display
    let error = ElfError::InvalidMagic;
    let error_str = error.to_string();
    assert!(error_str.contains("Invalid ELF magic number"), 
            "Error message should be descriptive");
    kprintln!("  ✅ Error display working");
    
    // Test error equality
    let error1 = ElfError::InvalidClass;
    let error2 = ElfError::InvalidClass;
    let error3 = ElfError::InvalidMagic;
    
    assert_eq!(error1, error2, "Same errors should be equal");
    assert_ne!(error1, error3, "Different errors should not be equal");
    kprintln!("  ✅ Error equality working");
    
    // Test error cloning
    let cloned_error = error1.clone();
    assert_eq!(error1, cloned_error, "Cloned error should be equal");
    kprintln!("  ✅ Error cloning working");
    
    kprintln!("✅ ELF error handling tests passed!");
    Ok(())
}

/// Test edge cases and boundary conditions
pub fn test_elf_edge_cases() -> Result<(), &'static str> {
    kprintln!("Testing ELF edge cases...");
    
    // Test exactly minimum size data
    let min_size_data = vec![0; 64]; // Exactly 64 bytes
    let result = Elf64Header::parse(&min_size_data);
    // This should fail due to invalid magic, not size
    assert!(matches!(result, Err(ElfError::InvalidMagic)), 
            "Minimum size data with invalid magic should fail on magic, not size");
    kprintln!("  ✅ Minimum size handling correct");
    
    // Test data larger than header
    let large_data = vec![0; 128]; // Larger than needed
    let result = Elf64Header::parse(&large_data);
    // This should fail due to invalid magic, not size
    assert!(matches!(result, Err(ElfError::InvalidMagic)), 
            "Large data with invalid magic should fail on magic, not size");
    kprintln!("  ✅ Large data handling correct");
    
    // Test header with all fields set to maximum values
    let mut test_data = create_test_elf64_header();
    // Set some fields to maximum values
    test_data[16..18].copy_from_slice(&0xFFFFu16.to_le_bytes()); // e_type = max
    test_data[18..20].copy_from_slice(&0xFFFFu16.to_le_bytes()); // e_machine = max
    
    let header = Elf64Header::parse(&test_data)?;
    // These should be parsed correctly even if they represent invalid values
    assert_eq!(header.e_type, 0xFFFF, "Maximum e_type should be parsed correctly");
    assert_eq!(header.e_machine, 0xFFFF, "Maximum e_machine should be parsed correctly");
    kprintln!("  ✅ Maximum value handling correct");
    
    kprintln!("✅ ELF edge cases tests passed!");
    Ok(())
}

/// Run all ELF header parser tests
pub fn run_all_elf_parser_tests() -> Result<(), &'static str> {
    kprintln!("🚀 Running ELF header parser tests...");
    
    test_elf64_header_basic()?;
    test_valid_elf64_header_parsing()?;
    test_invalid_elf_header_rejection()?;
    test_elf_header_validation()?;
    test_elf_enum_constants()?;
    test_elf_error_handling()?;
    test_elf_edge_cases()?;
    
    kprintln!("🎉 All ELF header parser tests completed successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic() {
        assert!(test_elf64_header_basic().is_ok());
    }
    
    #[test]
    fn test_valid_parsing() {
        assert!(test_valid_elf64_header_parsing().is_ok());
    }
    
    #[test]
    fn test_invalid_rejection() {
        assert!(test_invalid_elf_header_rejection().is_ok());
    }
    
    #[test]
    fn test_validation() {
        assert!(test_elf_header_validation().is_ok());
    }
    
    #[test]
    fn test_enum_constants() {
        assert!(test_elf_enum_constants().is_ok());
    }
    
    #[test]
    fn test_error_handling() {
        assert!(test_elf_error_handling().is_ok());
    }
    
    #[test]
    fn test_edge_cases() {
        assert!(test_elf_edge_cases().is_ok());
    }
    
    #[test]
    fn test_all() {
        assert!(run_all_elf_parser_tests().is_ok());
    }
}


