//! eBPF System Module
//!
//! Provides a safe, sandboxed execution environment for kernel extensions.

pub mod isa;
pub mod vm;
pub mod verifier;
pub mod trace;

pub use vm::Vm;
pub use verifier::verify;

/// Execute an eBPF program with verification
pub fn execute_program(program: &[u8]) -> Result<u64, vm::VmError> {
    // 1. Verify
    if let Err(_e) = verifier::verify(program) {
        // Map verifier error to VM error or just generic
        return Err(vm::VmError::InvalidInstruction); 
    }
    
    // 2. Execute
    let mut vm = Vm::new();
    vm.execute(program)
}
