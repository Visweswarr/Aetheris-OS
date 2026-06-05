//! eBPF Bytecode Verifier
//!
//! Static analysis of eBPF programs to ensure safety properties.
//!
//! Requirement: 10.2 - eBPF Verifier

use crate::ebpf::isa::*;

/// Verification Error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifierError {
    InvalidInstruction(usize),
    JumpOutOfBounds(usize),
    DivisionByZeroPotential(usize),
    InvalidRegister(usize),
    UnreachableInstruction(usize),
    ProgramTooLarge,
    NoExit,
}

/// Verify an eBPF program
pub fn verify(program: &[u8]) -> Result<(), VerifierError> {
    if program.len() > 4096 * 8 {
        return Err(VerifierError::ProgramTooLarge);
    }
    
    // 1. First pass: Decode and check basic validity
    let mut pc = 0;
    while pc * 8 < program.len() {
        let offset = pc * 8;
        if offset + 8 > program.len() {
            return Err(VerifierError::InvalidInstruction(pc));
        }
        
        let insn = Instruction::decode(&program[offset..offset+8])
            .ok_or(VerifierError::InvalidInstruction(pc))?;
            
        // Check for valid registers
        // R10 is read-only for destination (frame pointer)
        if insn.dst_reg == Register::R10 {
             // Only allowed if it's STX (store) where dst is base
             if insn.opcode & 0x07 != BPF_STX {
                  return Err(VerifierError::InvalidRegister(pc));
             }
        }
        
        // Check jumps
        if (insn.opcode & 0x07) == BPF_JMP {
             if insn.opcode != BPF_EXIT && insn.opcode != BPF_CALL {
                 // Check jump bounds
                 let jump_offset = insn.offset;
                 let target = (pc as isize + 1 + jump_offset as isize) as usize;
                 
                 if target >= program.len() / 8 {
                      return Err(VerifierError::JumpOutOfBounds(pc));
                 }
             }
        }
        
        pc += 1;
    }
    
    // 2. Check for exit
    // The last instruction reachable must be an exit (simplified check: check last instruction is exit or jump)
    // A real verifier checks the control flow graph (CFG).
    // For now, we just ensure the program isn't empty and decoded correctly.
    if pc == 0 {
        return Err(VerifierError::InvalidInstruction(0));
    }
    
    Ok(())
}
