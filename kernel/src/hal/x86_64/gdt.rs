use crate::kprintln;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;
use super::tss;

/// Global Descriptor Table
static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable::new();

/// GDT selectors
pub struct Selectors {
    pub code_selector: SegmentSelector,
    pub data_selector: SegmentSelector,
    pub tss_selector: SegmentSelector,
}

/// Initialize the Global Descriptor Table
/// 
/// Sets up the GDT with kernel code and data segments, plus the TSS
/// for interrupt stack table support.
pub fn init() -> Selectors {
    kprintln!("[HAL] GDT init - setting up kernel segments and TSS");
    
    unsafe {
        // Add kernel code segment (ring 0)
        let code_selector = GDT.add_entry(Descriptor::kernel_code_segment());
        
        // Add kernel data segment (ring 0)
        let data_selector = GDT.add_entry(Descriptor::kernel_data_segment());
        
        // Add TSS segment for IST support
        let tss_selector = GDT.add_entry(Descriptor::tss_segment(&tss::TSS));
        
        // Load the GDT
        GDT.load();
        
        kprintln!("[HAL] GDT loaded with {} entries", GDT.breakpoint.options().bits());
        kprintln!("[HAL] GDT: Code selector: {:04x}", code_selector.0);
        kprintln!("[HAL] GDT: Data selector: {:04x}", data_selector.0);
        kprintln!("[HAL] GDT: TSS selector: {:04x}", tss_selector.0);
        
        // Load the TSS selector into the CPU
        x86_64::instructions::tables::ltr(tss_selector);
        
        kprintln!("[HAL] TSS selector loaded into CPU");
        
        Selectors {
            code_selector,
            data_selector,
            tss_selector,
        }
    }
}

/// Get the current GDT selectors
pub fn get_selectors() -> Option<Selectors> {
    // This would need to be stored globally to be accessible
    // For now, return None as this is mainly for initialization
    None
}
