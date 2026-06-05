//! eBPF Instruction Set Architecture
//!
//! Defines the instructions, registers, and definitions for the eBPF VM.

/// eBPF Register
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Register {
    R0 = 0,  // Return value
    R1 = 1,  // Argument 1
    R2 = 2,  // Argument 2
    R3 = 3,  // Argument 3
    R4 = 4,  // Argument 4
    R5 = 5,  // Argument 5
    R6 = 6,  // Callee saved
    R7 = 7,  // Callee saved
    R8 = 8,  // Callee saved
    R9 = 9,  // Callee saved
    R10 = 10, // Frame pointer (read-only)
}

impl Register {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Register::R0),
            1 => Some(Register::R1),
            2 => Some(Register::R2),
            3 => Some(Register::R3),
            4 => Some(Register::R4),
            5 => Some(Register::R5),
            6 => Some(Register::R6),
            7 => Some(Register::R7),
            8 => Some(Register::R8),
            9 => Some(Register::R9),
            10 => Some(Register::R10),
            _ => None,
        }
    }
}

/// eBPF Instruction classes
pub const BPF_LD: u8 = 0x00;
pub const BPF_LDX: u8 = 0x01;
pub const BPF_ST: u8 = 0x02;
pub const BPF_STX: u8 = 0x03;
pub const BPF_ALU: u8 = 0x04;
pub const BPF_JMP: u8 = 0x05;
pub const BPF_JMP32: u8 = 0x06;
pub const BPF_ALU64: u8 = 0x07;

/// eBPF OpCodes
pub const BPF_ADD: u8 = 0x00;
pub const BPF_SUB: u8 = 0x10;
pub const BPF_MUL: u8 = 0x20;
pub const BPF_DIV: u8 = 0x30;
pub const BPF_OR: u8 = 0x40;
pub const BPF_AND: u8 = 0x50;
pub const BPF_LSH: u8 = 0x60;
pub const BPF_RSH: u8 = 0x70;
pub const BPF_NEG: u8 = 0x80;
pub const BPF_MOD: u8 = 0x90;
pub const BPF_XOR: u8 = 0xa0;
pub const BPF_MOV: u8 = 0xb0;
pub const BPF_ARSH: u8 = 0xc0;
pub const BPF_END: u8 = 0xd0;

pub const BPF_JA: u8 = 0x00;
pub const BPF_JEQ: u8 = 0x10;
pub const BPF_JGT: u8 = 0x20;
pub const BPF_JGE: u8 = 0x30;
pub const BPF_JSET: u8 = 0x40;
pub const BPF_JNE: u8 = 0x50;
pub const BPF_JSGT: u8 = 0x60;
pub const BPF_JSGE: u8 = 0x70;
pub const BPF_CALL: u8 = 0x80;
pub const BPF_EXIT: u8 = 0x90;
pub const BPF_JLT: u8 = 0xa0;
pub const BPF_JLE: u8 = 0xb0;
pub const BPF_JSLT: u8 = 0xc0;
pub const BPF_JSLE: u8 = 0xd0;

/// Source operands
pub const BPF_K: u8 = 0x00; // Immediate
pub const BPF_X: u8 = 0x08; // Register

/// Size modifiers
pub const BPF_W: u8 = 0x00; // Word (4 bytes)
pub const BPF_H: u8 = 0x08; // Half word (2 bytes)
pub const BPF_B: u8 = 0x10; // Byte (1 byte)
pub const BPF_DW: u8 = 0x18; // Double word (8 bytes)

/// Decoded Instruction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instruction {
    pub opcode: u8,
    pub dst_reg: Register,
    pub src_reg: Register,
    pub offset: i16,
    pub imm: i32,
}

impl Instruction {
    /// Decode a 64-bit instruction from bytes
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 8 {
            return None;
        }
        
        // Helper to read u8
        let opcode = bytes[0];
        // Registers are in byte 1: src (high 4), dst (low 4)
        let regs = bytes[1];
        let dst_idx = regs & 0x0F;
        let src_idx = (regs >> 4) & 0x0F;
        
        let dst_reg = Register::from_u8(dst_idx)?;
        let src_reg = Register::from_u8(src_idx)?;
        
        // Offset is bytes 2-3 (little endian)
        let offset = ((bytes[3] as u16) << 8 | (bytes[2] as u16)) as i16;
        
        // Immediate is bytes 4-7 (little endian)
        let imm = ((bytes[7] as u32) << 24 | (bytes[6] as u32) << 16 | 
                   (bytes[5] as u32) << 8 | (bytes[4] as u32)) as i32;
                   
        Some(Self {
            opcode,
            dst_reg,
            src_reg,
            offset,
            imm,
        })
    }
}
