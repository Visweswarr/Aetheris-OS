//! eBPF Virtual Machine
//!
//! Executes eBPF bytecode in a safe, sandboxed environment.

use alloc::vec::Vec;
use crate::ebpf::isa::*;

/// Stack size in bytes (512 bytes is standard for eBPF)
pub const STACK_SIZE: usize = 512;

/// VM execution error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmError {
    InvalidInstruction,
    DivisionByZero,
    MemoryOutOfBounds,
    StackOverflow,
    HelperFunctionError,
    UnknownOpcode(u8),
    InvalidRegister,
}

/// eBPF Virtual Machine State
pub struct Vm {
    /// Registers R0-R10
    pub registers: [u64; 11],
    /// Stack memory
    pub stack: [u8; STACK_SIZE],
    /// Program Counter
    pub pc: usize,
}

impl Vm {
    /// Create a new VM instance
    pub fn new() -> Self {
        let mut vm = Self {
            registers: [0; 11],
            stack: [0; STACK_SIZE],
            pc: 0,
        };
        // R10 is the frame pointer, points to top of stack
        // In eBPF, stack grows down. R10 points to end of stack buffer.
        // We'll use a virtual address or just offset 0 for now?
        // Standard eBPF uses R10 as (stack_ptr + 512).
        // For simplicity, we'll handle stack access via R10 as relative to `stack` array.
        // But eBPF semantics are: [R10 - offset].
        // So R10 should be set to "end of stack".
        vm.registers[10] = STACK_SIZE as u64; 
        vm
    }

    /// Execute a program
    /// 
    /// Returns the value in R0 upon BPF_EXIT
    pub fn execute(&mut self, program: &[u8]) -> Result<u64, VmError> {
        let iterations_max = 100_000; // Sanity limit for loops (if verifier missed something)
        let mut instructions_executed = 0;

        while self.pc * 8 < program.len() {
            if instructions_executed >= iterations_max {
                // Should encounter exit before this
                return Err(VmError::StackOverflow); // Abuse error for cycle limit
            }
            instructions_executed += 1;

            let pc_offset = self.pc * 8;
            if pc_offset + 8 > program.len() {
                return Err(VmError::InvalidInstruction);
            }

            let insn_bytes = &program[pc_offset..pc_offset+8];
            let insn = Instruction::decode(insn_bytes).ok_or(VmError::InvalidInstruction)?;
            
            self.pc += 1; // Advance PC by default

            self.execute_instruction(insn)?;
            
            if insn.opcode == BPF_JMP | BPF_EXIT {
                 return Ok(self.registers[0]);
            }
        }
        
        // Fall off end of program is implementation defined, usually 0
        Ok(self.registers[0])
    }

    fn execute_instruction(&mut self, insn: Instruction) -> Result<(), VmError> {
        let opcode_class = insn.opcode & 0x07;
        let opcode = insn.opcode;
        
        let dst = insn.dst_reg as usize;
        let src = insn.src_reg as usize;
        
        match opcode_class {
            BPF_ALU64 => {
                let op = opcode & 0xF0;
                let src_val = if (opcode & 0x08) == 0 { // BPF_K
                    insn.imm as i64 as u64
                } else { // BPF_X
                    self.registers[src]
                };

                match op {
                    BPF_ADD => self.registers[dst] = self.registers[dst].wrapping_add(src_val),
                    BPF_SUB => self.registers[dst] = self.registers[dst].wrapping_sub(src_val),
                    BPF_MUL => self.registers[dst] = self.registers[dst].wrapping_mul(src_val),
                    BPF_DIV => {
                        if src_val == 0 { return Err(VmError::DivisionByZero); }
                        self.registers[dst] /= src_val;
                    }
                    BPF_OR  => self.registers[dst] |= src_val,
                    BPF_AND => self.registers[dst] &= src_val,
                    BPF_LSH => self.registers[dst] <<= src_val,
                    BPF_RSH => self.registers[dst] >>= src_val,
                    BPF_XOR => self.registers[dst] ^= src_val,
                    BPF_MOV => self.registers[dst] = src_val,
                    BPF_ARSH => {
                        // Arithmetic shift (preserve sign)
                        let val = self.registers[dst] as i64;
                        self.registers[dst] = (val >> src_val) as u64;
                    },
                    _ => return Err(VmError::UnknownOpcode(opcode)),
                }
            },
            BPF_JMP => {
                let op = opcode & 0xF0;
                if op == BPF_EXIT {
                    // Logic handled in loop
                    return Ok(());
                }
                
                // Jumps
                // All jumps use off (i16) except JA (calls/exits are special)
                // Wait, JA is unconditional jump with off.
                
                let jmp_target = (self.pc as isize + insn.offset as isize) as usize;
                let mut should_jump = false;
                
                if op == BPF_JA {
                    should_jump = true;
                } else if op == BPF_CALL {
                    // Helper call
                    let func_id = insn.imm;
                    self.call_helper(func_id)?;
                    return Ok(());
                } else {
                    let src_val = if (opcode & 0x08) == 0 { // BPF_K
                         insn.imm as i64 as u64
                    } else { // BPF_X
                        self.registers[src]
                    };
                    let dst_val = self.registers[dst];

                    match op {
                         BPF_JEQ => should_jump = dst_val == src_val,
                         BPF_JGT => should_jump = dst_val > src_val,
                         BPF_JGE => should_jump = dst_val >= src_val,
                         BPF_JSET => should_jump = (dst_val & src_val) != 0,
                         BPF_JNE => should_jump = dst_val != src_val,
                         BPF_JSGT => should_jump = (dst_val as i64) > (src_val as i64),
                         BPF_JSGE => should_jump = (dst_val as i64) >= (src_val as i64),
                         BPF_JLT => should_jump = dst_val < src_val,
                         BPF_JLE => should_jump = dst_val <= src_val,
                         BPF_JSLT => should_jump = (dst_val as i64) < (src_val as i64),
                         BPF_JSLE => should_jump = (dst_val as i64) <= (src_val as i64),
                         _ => return Err(VmError::UnknownOpcode(opcode)),
                    }
                }
                
                if should_jump {
                    self.pc = jmp_target;
                }
            },
            BPF_STX => {
                // *(size *)(dst + off) = src
                let addr = (self.registers[dst] as isize + insn.offset as isize) as usize;
                // Currently only supporting stack access
                // Addr must be within [0, STACK_SIZE)
                // Since registers[10] = STACK_SIZE, and typical access is [R10 - off]
                // Address logic needs to map to stack array index
                if addr >= STACK_SIZE {
                     return Err(VmError::MemoryOutOfBounds);
                }
                
                // Store src value to stack
                // Simplified byte store
                let size_mode = opcode & 0x18;
                let val = self.registers[src];
                
                match size_mode {
                    BPF_B => self.stack[addr] = val as u8,
                    BPF_DW => {
                        if addr + 8 > STACK_SIZE { return Err(VmError::MemoryOutOfBounds); }
                        let bytes = val.to_le_bytes();
                        self.stack[addr..addr+8].copy_from_slice(&bytes);
                    },
                    // Add other sizes
                    _ => unimplemented!("Size not implemented"),
                }
            },
            BPF_LDX => {
               // dst = *(size *)(src + off)
               // Similar logic
               let addr = (self.registers[src] as isize + insn.offset as isize) as usize;
                if addr >= STACK_SIZE {
                     return Err(VmError::MemoryOutOfBounds);
                }
                
                let size_mode = opcode & 0x18;
                match size_mode {
                    BPF_DW => {
                        if addr + 8 > STACK_SIZE { return Err(VmError::MemoryOutOfBounds); }
                        let mut bytes = [0u8; 8];
                        bytes.copy_from_slice(&self.stack[addr..addr+8]);
                        self.registers[dst] = u64::from_le_bytes(bytes);
                    },
                    _ => unimplemented!("Size not implemented"),
                }
            },
            // Other classes...
            _ => return Err(VmError::UnknownOpcode(opcode)),
        }
        
        Ok(())
    }
    
    fn call_helper(&mut self, func_id: i32) -> Result<(), VmError> {
        // Implement helper functions (print, get_time, etc.)
        match func_id {
            1 => { // trace_printk
                // R1 = fmt string, R2-R5 args
                // For now just logging "Help called"
                crate::kprintln!("[eBPF] Helper called!");
                self.registers[0] = 0;
            },
            _ => return Err(VmError::HelperFunctionError),
        }
        Ok(())
    }
}
