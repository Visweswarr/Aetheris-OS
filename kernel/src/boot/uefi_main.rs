//! UEFI entry point for Polymera OS kernel
//!
//! This module provides the main entry point when booting via UEFI.

#![no_main]
#![no_std]

extern crate alloc;

use alloc::{vec, vec::Vec};
use core::mem;
use uefi::prelude::*;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat as UefiPixelFormat};
use uefi::proto::loaded_image::LoadedImage;
use uefi::table::boot::{MemoryDescriptor, MemoryType as UefiMemoryType};

use polymera_kernel::{
    boot::{BootConfig, BootMethod, FramebufferInfo, PixelFormat, memory},
    KernelConfig, KernelInfo, MemoryRegion, MemoryType,
    kernel_early_init, kernel_main_init, kernel_main_loop,
    log, error::KernelResult,
};

/// UEFI entry point
#[no_mangle]
pub extern "efiapi" fn efi_main(image: Handle, mut system_table: SystemTable<Boot>) -> Status {
    // Initialize UEFI services
    uefi_services::init(&mut system_table).unwrap();
    
    // Set up early logging
    init_uefi_logging(&mut system_table);
    
    log::kprintln!("Polymera OS UEFI bootloader starting...");
    
    // Initialize the kernel
    match uefi_kernel_init(image, &mut system_table) {
        Ok(_) => {
            log::kprintln!("Kernel initialization successful, entering main loop");
            kernel_main_loop();
        }
        Err(e) => {
            log::kprintln!("Kernel initialization failed: {:?}", e);
            Status::ABORTED
        }
    }
}

/// Initialize kernel from UEFI environment
fn uefi_kernel_init(image: Handle, system_table: &mut SystemTable<Boot>) -> KernelResult<()> {
    log::kprintln!("Setting up UEFI environment...");
    
    // Get loaded image protocol
    let loaded_image = system_table
        .boot_services()
        .open_protocol_exclusive::<LoadedImage>(image)
        .map_err(|_| polymera_kernel::error::KernelError::BootloaderError)?;
    
    // Get memory map
    let memory_map = get_memory_map(system_table)?;
    
    // Get framebuffer info
    let framebuffer = get_framebuffer_info(system_table);
    
    // Create boot configuration
    let boot_config = BootConfig {
        boot_method: BootMethod::Uefi,
        framebuffer,
        serial_base: Some(0x3F8), // COM1 port
        acpi_rsdp: get_acpi_rsdp(system_table),
        device_tree: None, // Not used on x86_64
    };
    
    // Create kernel configuration
    let kernel_config = KernelConfig {
        boot_info: KernelInfo {
            name: polymera_kernel::KERNEL_NAME,
            version: polymera_kernel::KERNEL_VERSION,
            git_hash: polymera_kernel::GIT_HASH,
            build_date: polymera_kernel::BUILD_DATE,
            state: polymera_kernel::get_kernel_state(),
        },
        memory_regions: memory_map,
        cmdline: get_command_line(&loaded_image),
        debug_mode: true, // Enable debug mode for now
    };
    
    log::kprintln!("Memory regions: {} found", kernel_config.memory_regions.len());
    log::kprintln!("Total usable memory: {} MB", 
        memory::total_usable_memory(&kernel_config.memory_regions) / 1024 / 1024);
    
    // Exit UEFI boot services
    log::kprintln!("Exiting UEFI boot services...");
    let memory_map_size = system_table.boot_services().memory_map_size();
    let mut memory_map_buffer = vec![0u8; memory_map_size.map_size + 2 * memory_map_size.entry_size];
    
    let (system_table, _memory_map) = system_table
        .exit_boot_services(image, &mut memory_map_buffer)
        .map_err(|_| polymera_kernel::error::KernelError::BootloaderError)?;
    
    // We now have control of the system - UEFI boot services are no longer available
    log::kprintln!("UEFI boot services exited successfully");
    
    // Initialize the kernel
    kernel_early_init(kernel_config.clone())?;
    kernel_main_init(kernel_config)?;
    
    Ok(())
}

/// Get system memory map from UEFI
fn get_memory_map(system_table: &SystemTable<Boot>) -> KernelResult<Vec<MemoryRegion>> {
    let boot_services = system_table.boot_services();
    let memory_map_size = boot_services.memory_map_size();
    let mut memory_map_buffer = vec![0u8; memory_map_size.map_size + 2 * memory_map_size.entry_size];
    
    let memory_map = boot_services
        .memory_map(&mut memory_map_buffer)
        .map_err(|_| polymera_kernel::error::KernelError::MemoryError)?;
    
    let mut regions = Vec::new();
    
    for descriptor in memory_map.entries() {
        let region_type = match descriptor.ty {
            UefiMemoryType::CONVENTIONAL => MemoryType::Usable,
            UefiMemoryType::LOADER_CODE | UefiMemoryType::LOADER_DATA => MemoryType::Bootloader,
            UefiMemoryType::RUNTIME_SERVICES_CODE | UefiMemoryType::RUNTIME_SERVICES_DATA => MemoryType::Reserved,
            UefiMemoryType::ACPI_RECLAIM => MemoryType::AcpiReclaimable,
            UefiMemoryType::ACPI_NON_VOLATILE => MemoryType::AcpiNvs,
            UefiMemoryType::UNUSABLE => MemoryType::BadMemory,
            _ => MemoryType::Reserved,
        };
        
        regions.push(MemoryRegion {
            start: descriptor.phys_start,
            size: descriptor.page_count * 4096, // 4KB pages
            region_type,
        });
    }
    
    Ok(regions)
}

/// Get framebuffer information from UEFI GOP
fn get_framebuffer_info(system_table: &SystemTable<Boot>) -> Option<FramebufferInfo> {
    let boot_services = system_table.boot_services();
    
    // Try to get Graphics Output Protocol
    if let Ok(gop) = boot_services.locate_protocol::<GraphicsOutput>() {
        let gop = unsafe { &mut *gop.get() };
        let mode_info = gop.current_mode_info();
        let framebuffer = gop.frame_buffer();
        
        let pixel_format = match mode_info.pixel_format() {
            UefiPixelFormat::Rgb => PixelFormat::Rgb,
            UefiPixelFormat::Bgr => PixelFormat::Bgr,
            UefiPixelFormat::Bitmask => PixelFormat::Bitmask,
            UefiPixelFormat::BltOnly => PixelFormat::BltOnly,
        };
        
        Some(FramebufferInfo {
            base_addr: framebuffer.as_mut_ptr() as u64,
            width: mode_info.resolution().0 as u32,
            height: mode_info.resolution().1 as u32,
            bytes_per_pixel: 4, // Assuming 32-bit pixels
            stride: mode_info.stride() as u32 * 4,
            pixel_format,
        })
    } else {
        None
    }
}

/// Get ACPI RSDP from UEFI configuration tables
fn get_acpi_rsdp(system_table: &SystemTable<Boot>) -> Option<u64> {
    use uefi::table::cfg;
    
    for entry in system_table.config_table() {
        if entry.guid == cfg::ACPI2_GUID || entry.guid == cfg::ACPI_GUID {
            return Some(entry.address as u64);
        }
    }
    None
}

/// Get command line from loaded image
fn get_command_line(loaded_image: &LoadedImage) -> Option<alloc::string::String> {
    loaded_image.load_options_as_cstr16()
        .ok()
        .and_then(|options| options.to_string().ok())
}

/// Initialize UEFI logging
fn init_uefi_logging(system_table: &mut SystemTable<Boot>) {
    // Set up serial port for debugging if available
    // This is a simplified implementation
    log::kprintln!("UEFI logging initialized");
}

/// UEFI-specific implementation for boot module
pub mod uefi {
    use super::*;
    use crate::error::KernelResult;
    
    /// Convert UEFI memory map to kernel memory regions
    pub fn convert_uefi_memory_map(map_data: &[u8]) -> KernelResult<Vec<MemoryRegion>> {
        // This would parse the raw UEFI memory map data
        // For now, return empty vector as placeholder
        Ok(Vec::new())
    }
    
    /// Initialize UEFI logging
    pub fn init_uefi_logging() -> KernelResult<()> {
        // Initialize UEFI-specific logging
        Ok(())
    }
}

/// Simple serial port implementation for early debugging
struct UefiSerialPort {
    base: u16,
}

impl UefiSerialPort {
    fn new(base: u16) -> Self {
        Self { base }
    }
    
    fn write_byte(&self, byte: u8) {
        unsafe {
            // Wait for transmit buffer to be ready
            while (x86_64::instructions::port::Port::new(self.base + 5).read() & 0x20) == 0 {}
            
            // Write byte to transmit buffer
            x86_64::instructions::port::Port::new(self.base).write(byte);
        }
    }
}

impl polymera_kernel::boot::SerialPort for UefiSerialPort {
    fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }
}