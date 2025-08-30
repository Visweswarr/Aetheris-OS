//! Execution Module
//! 
//! This module provides functionality for executing programs,
//! including ELF file parsing and program loading.

pub mod elf;
pub mod header;
pub mod loader;

/// Test function for execution functionality
pub fn test_exec() -> Result<(), &'static str> {
    kprintln!("Testing execution functionality...");
    
    // Test ELF header parsing
    let test_header = elf::create_test_elf64_header();
    let header = elf::Elf64Header::parse(&test_header)
        .map_err(|_| "Failed to parse valid ELF64 header")?;
    
    // Verify header properties
    if header.get_class()? != elf::ElfClass::Elf64 {
        return Err("Invalid ELF class");
    }
    
    if header.get_file_type()? != elf::ElfType::Executable {
        return Err("Invalid file type");
    }
    
    if header.get_machine()? != elf::ElfMachine::X86_64 {
        return Err("Invalid machine architecture");
    }
    
    // Test invalid header parsing
    let invalid_header = elf::create_invalid_elf_header();
    let result = elf::Elf64Header::parse(&invalid_header);
    if result.is_ok() {
        return Err("Invalid header should not parse successfully");
    }
    
    // Test ELF32 header rejection
    let elf32_header = elf::create_elf32_header();
    let result = elf::Elf64Header::parse(&elf32_header);
    if result.is_ok() {
        return Err("ELF32 header should not parse as ELF64");
    }
    
    kprintln!("✅ Execution functionality tests passed!");
    Ok(())
}


