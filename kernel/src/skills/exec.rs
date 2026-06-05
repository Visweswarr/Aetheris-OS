use crate::{kprintln, klog, vec};
use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

use crate::skills::manifest::SkillManifestV1;
use crate::skills::wasi::WasiHost;
use crate::intent::schema::{PlanPreviewV1, ActionV1, EvidenceV1};

pub const MAX_INSTRUCTIONS: u64 = 1_000_000; // 1M instructions
pub const MAX_MEMORY_BYTES: usize = 32 * 1024 * 1024; // 32 MiB
pub const MAX_TIME_MS: u64 = 250; // 250ms

#[derive(Debug)]
pub struct ExecutionMetrics {
    pub instructions_executed: u64,
    pub memory_used_bytes: usize,
    pub execution_time_us: u64,
    pub hostcalls_made: u32,
}

impl ExecutionMetrics {
    pub fn new() -> Self {
        Self {
            instructions_executed: 0,
            memory_used_bytes: 0,
            execution_time_us: 0,
            hostcalls_made: 0,
        }
    }
    
    pub fn add_instructions(&mut self, count: u64) {
        self.instructions_executed += count;
    }
    
    pub fn set_memory_usage(&mut self, bytes: usize) {
        self.memory_used_bytes = bytes;
    }
    
    pub fn set_execution_time(&mut self, us: u64) {
        self.execution_time_us = us;
    }
    
    pub fn increment_hostcalls(&mut self) {
        self.hostcalls_made += 1;
    }
}

#[derive(Debug)]
pub struct PreviewBundle {
    pub preview: PlanPreviewV1,
    pub evidence: Vec<EvidenceV1>,
    pub metrics: ExecutionMetrics,
}

impl PreviewBundle {
    pub fn new(preview: PlanPreviewV1, evidence: Vec<EvidenceV1>, metrics: ExecutionMetrics) -> Self {
        Self {
            preview,
            evidence,
            metrics,
        }
    }
}

#[derive(Debug)]
pub struct Skill {
    pub id: u64,
    pub manifest: SkillManifestV1,
    pub wasm_bytes: Vec<u8>,
    pub wasi_host: WasiHost,
    pub memory: Vec<u8>,
    pub stack: Vec<u32>,
    pub instruction_counter: AtomicU64,
    pub start_time: u64,
}

impl Skill {
    pub fn new(id: u64, manifest: SkillManifestV1, wasm_bytes: Vec<u8>) -> Self {
        let wasi_host = WasiHost::new(id);
        let memory = Vec::with_capacity(manifest.mem_limit_bytes as usize);
        let stack = Vec::new();
        
        Self {
            id,
            manifest,
            wasm_bytes,
            wasi_host,
            memory,
            stack,
            instruction_counter: AtomicU64::new(0),
            start_time: crate::time::get_current_time_ms(),
        }
    }
    
    pub fn run_preview(&mut self, input: &[u8]) -> Result<PreviewBundle, ExecutionError> {
        let start_time = crate::time::get_current_time_ms();
        
        self.instruction_counter.store(0, Ordering::SeqCst);
        self.memory.clear();
        self.stack.clear();
        
        let mut metrics = ExecutionMetrics::new();
        
        match self.execute_wasm(input) {
            Ok(_) => {
                let end_time = crate::time::get_current_time_ms();
                let execution_time = (end_time - start_time) * 1000; // Convert to microseconds
                
                metrics.set_execution_time(execution_time);
                metrics.set_memory_usage(self.memory.len());
                metrics.instructions_executed = self.instruction_counter.load(Ordering::SeqCst);
                
                let context = self.wasi_host.get_context();
                
                let preview = PlanPreviewV1 {
                    plan: crate::intent::schema::PlanV1 {
                        intent_id: 0, // Will be set by caller
                        actions: context.plan_actions,
                        cost: metrics.instructions_executed as u64,
                        total_cost: metrics.instructions_executed as u64,
                        total_time: execution_time / 1000,
                        constraints_applied: Vec::new(),
                    },
                    risks: Vec::new(),
                    notes: vec![
                        format!("Skill {} executed in {}μs", self.manifest.name, execution_time),
                        format!("Used {} bytes of memory", self.memory.len()),
                        format!("Executed {} instructions", metrics.instructions_executed),
                    ],
                    confidence: 100,
                };
                
                let bundle = PreviewBundle::new(preview, context.evidence, metrics);
                Ok(bundle)
            }
            Err(e) => {
                let end_time = crate::time::get_current_time_ms();
                let execution_time = (end_time - start_time) * 1000;
                
                metrics.set_execution_time(execution_time);
                metrics.set_memory_usage(self.memory.len());
                metrics.instructions_executed = self.instruction_counter.load(Ordering::SeqCst);
                
                Err(e)
            }
        }
    }
    
    fn execute_wasm(&mut self, input: &[u8]) -> Result<(), ExecutionError> {
        if self.wasm_bytes.len() < 8 {
            return Err(ExecutionError::InvalidWasm);
        }
        
        if !self.validate_wasm_header() {
            return Err(ExecutionError::InvalidWasm);
        }
        
        self.setup_memory(input);
        self.execute_wasm_instructions()?;
        
        Ok(())
    }
    
    fn validate_wasm_header(&self) -> bool {
        if self.wasm_bytes.len() < 8 {
            return false;
        }
        
        let magic = &self.wasm_bytes[0..4];
        let version = &self.wasm_bytes[4..8];
        
        magic == b"\x00asm" && version == b"\x01\x00\x00\x00"
    }
    
    fn setup_memory(&mut self, input: &[u8]) {
        self.memory.clear();
        
        let initial_memory = 64 * 1024; // 64 KiB initial
        self.memory.resize(initial_memory, 0);
        
        if input.len() > 0 {
            let input_offset = 1024; // Start at 1KB
            if input_offset + input.len() <= self.memory.len() {
                self.memory[input_offset..input_offset + input.len()].copy_from_slice(input);
            }
        }
    }
    
    fn execute_wasm_instructions(&mut self) -> Result<(), ExecutionError> {
        let mut pc = 8; // Skip header
        
        while pc < self.wasm_bytes.len() {
            if self.instruction_counter.load(Ordering::SeqCst) >= MAX_INSTRUCTIONS {
                return Err(ExecutionError::InstructionLimitExceeded);
            }
            
            if self.memory.len() > MAX_MEMORY_BYTES {
                return Err(ExecutionError::MemoryLimitExceeded);
            }
            
            let current_time = crate::time::get_current_time_ms();
            if current_time - self.start_time > MAX_TIME_MS {
                return Err(ExecutionError::TimeLimitExceeded);
            }
            
            match self.execute_next_instruction(&mut pc) {
                Ok(_) => {
                    self.instruction_counter.fetch_add(1, Ordering::SeqCst);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        
        Ok(())
    }
    
    fn execute_next_instruction(&mut self, pc: &mut usize) -> Result<(), ExecutionError> {
        if *pc >= self.wasm_bytes.len() {
            return Err(ExecutionError::UnexpectedEnd);
        }
        
        let opcode = self.wasm_bytes[*pc];
        *pc += 1;
        
        match opcode {
            0x00 => self.op_unreachable(),
            0x01 => self.op_nop(),
            0x02 => self.op_block(pc),
            0x03 => self.op_loop(pc),
            0x04 => self.op_if(pc),
            0x05 => self.op_else(),
            0x0B => self.op_end(),
            0x0C => self.op_br(pc),
            0x0D => self.op_br_if(pc),
            0x20 => self.op_local_get(pc),
            0x21 => self.op_local_set(pc),
            0x22 => self.op_local_tee(pc),
            0x23 => self.op_global_get(pc),
            0x24 => self.op_global_set(pc),
            0x41 => self.op_i32_const(pc),
            0x42 => self.op_i64_const(pc),
            0x43 => self.op_f32_const(pc),
            0x44 => self.op_f64_const(pc),
            0x6A => self.op_i32_add(),
            0x6B => self.op_i32_sub(),
            0x6C => self.op_i32_mul(),
            0x6D => self.op_i32_div_s(),
            0x6E => self.op_i32_div_u(),
            0x6F => self.op_i32_rem_s(),
            0x70 => self.op_i32_rem_u(),
            0xFC => self.op_call_host(pc),
            _ => {
                klog!(ERROR, "[SKILL-{}] Unknown opcode: 0x{:02X}", self.id, opcode);
                return Err(ExecutionError::UnknownOpcode(opcode));
            }
        }
    }
    
    fn op_unreachable(&mut self) -> Result<(), ExecutionError> {
        Err(ExecutionError::Unreachable)
    }
    
    fn op_nop(&mut self) -> Result<(), ExecutionError> {
        Ok(())
    }
    
    fn op_block(&mut self, _pc: &mut usize) -> Result<(), ExecutionError> {
        self.stack.push(0); // Block marker
        Ok(())
    }
    
    fn op_loop(&mut self, _pc: &mut usize) -> Result<(), ExecutionError> {
        self.stack.push(1); // Loop marker
        Ok(())
    }
    
    fn op_if(&mut self, _pc: &mut usize) -> Result<(), ExecutionError> {
        if let Some(condition) = self.stack.pop() {
            if condition != 0 {
                self.stack.push(2); // If marker
            } else {
                self.stack.push(3); // Else marker
            }
        }
        Ok(())
    }
    
    fn op_else(&mut self) -> Result<(), ExecutionError> {
        if let Some(marker) = self.stack.pop() {
            if marker == 2 {
                self.stack.push(3); // Convert if to else
            }
        }
        Ok(())
    }
    
    fn op_end(&mut self) -> Result<(), ExecutionError> {
        while let Some(marker) = self.stack.pop() {
            if marker < 2 {
                break;
            }
        }
        Ok(())
    }
    
    fn op_br(&mut self, _pc: &mut usize) -> Result<(), ExecutionError> {
        if let Some(_depth) = self.stack.pop() {
            // Simplified branch implementation
        }
        Ok(())
    }
    
    fn op_br_if(&mut self, _pc: &mut usize) -> Result<(), ExecutionError> {
        if let (Some(_depth), Some(condition)) = (self.stack.pop(), self.stack.pop()) {
            if condition != 0 {
                // Simplified conditional branch
            }
        }
        Ok(())
    }
    
    fn op_local_get(&mut self, _pc: &mut usize) -> Result<(), ExecutionError> {
        self.stack.push(0); // Simplified local variable access
        Ok(())
    }
    
    fn op_local_set(&mut self, _pc: &mut usize) -> Result<(), ExecutionError> {
        self.stack.pop(); // Simplified local variable assignment
        Ok(())
    }
    
    fn op_local_tee(&mut self, _pc: &mut usize) -> Result<(), ExecutionError> {
        if let Some(value) = self.stack.last().copied() {
            self.stack.push(value);
        }
        Ok(())
    }
    
    fn op_global_get(&mut self, _pc: &mut usize) -> Result<(), ExecutionError> {
        self.stack.push(0); // Simplified global variable access
        Ok(())
    }
    
    fn op_global_set(&mut self, _pc: &mut usize) -> Result<(), ExecutionError> {
        self.stack.pop(); // Simplified global variable assignment
        Ok(())
    }
    
    fn op_i32_const(&mut self, pc: &mut usize) -> Result<(), ExecutionError> {
        let value = self.read_leb128_u32(pc)?;
        self.stack.push(value);
        Ok(())
    }
    
    fn op_i64_const(&mut self, pc: &mut usize) -> Result<(), ExecutionError> {
        let value = self.read_leb128_u64(pc)?;
        self.stack.push((value & 0xFFFFFFFF) as u32);
        Ok(())
    }
    
    fn op_f32_const(&mut self, pc: &mut usize) -> Result<(), ExecutionError> {
        if *pc + 4 > self.wasm_bytes.len() {
            return Err(ExecutionError::UnexpectedEnd);
        }
        let bytes = [
            self.wasm_bytes[*pc],
            self.wasm_bytes[*pc + 1],
            self.wasm_bytes[*pc + 2],
            self.wasm_bytes[*pc + 3],
        ];
        let value = f32::from_le_bytes(bytes);
        self.stack.push(value.to_bits());
        *pc += 4;
        Ok(())
    }
    
    fn op_f64_const(&mut self, pc: &mut usize) -> Result<(), ExecutionError> {
        if *pc + 8 > self.wasm_bytes.len() {
            return Err(ExecutionError::UnexpectedEnd);
        }
        let bytes = [
            self.wasm_bytes[*pc],
            self.wasm_bytes[*pc + 1],
            self.wasm_bytes[*pc + 2],
            self.wasm_bytes[*pc + 3],
            self.wasm_bytes[*pc + 4],
            self.wasm_bytes[*pc + 5],
            self.wasm_bytes[*pc + 6],
            self.wasm_bytes[*pc + 7],
        ];
        let value = f64::from_le_bytes(bytes);
        self.stack.push((value.to_bits() & 0xFFFFFFFF) as u32);
        *pc += 8;
        Ok(())
    }
    
    fn op_i32_add(&mut self) -> Result<(), ExecutionError> {
        if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
            self.stack.push(a.wrapping_add(b));
        }
        Ok(())
    }
    
    fn op_i32_sub(&mut self) -> Result<(), ExecutionError> {
        if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
            self.stack.push(a.wrapping_sub(b));
        }
        Ok(())
    }
    
    fn op_i32_mul(&mut self) -> Result<(), ExecutionError> {
        if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
            self.stack.push(a.wrapping_mul(b));
        }
        Ok(())
    }
    
    fn op_i32_div_s(&mut self) -> Result<(), ExecutionError> {
        if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
            if b == 0 {
                return Err(ExecutionError::DivisionByZero);
            }
            self.stack.push((a as i32 / b as i32) as u32);
        }
        Ok(())
    }
    
    fn op_i32_div_u(&mut self) -> Result<(), ExecutionError> {
        if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
            if b == 0 {
                return Err(ExecutionError::DivisionByZero);
            }
            self.stack.push(a / b);
        }
        Ok(())
    }
    
    fn op_i32_rem_s(&mut self) -> Result<(), ExecutionError> {
        if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
            if b == 0 {
                return Err(ExecutionError::DivisionByZero);
            }
            self.stack.push((a as i32 % b as i32) as u32);
        }
        Ok(())
    }
    
    fn op_i32_rem_u(&mut self) -> Result<(), ExecutionError> {
        if let (Some(b), Some(a)) = (self.stack.pop(), self.stack.pop()) {
            if b == 0 {
                return Err(ExecutionError::DivisionByZero);
            }
            self.stack.push(a % b);
        }
        Ok(())
    }
    
    fn op_call_host(&mut self, pc: &mut usize) -> Result<(), ExecutionError> {
        let hostcall_id = self.read_leb128_u32(pc)?;
        let param_count = self.read_leb128_u32(pc)?;
        
        let mut params = Vec::new();
        for _ in 0..param_count {
            if let Some(value) = self.stack.pop() {
                params.push(value);
            }
        }
        
        params.reverse();
        
        match self.wasi_host.handle_hostcall(hostcall_id, &params) {
            Ok(result) => {
                self.stack.push(result);
            }
            Err(e) => {
                klog!(ERROR, "[SKILL-{}] Hostcall failed: {:?}", self.id, e);
                self.stack.push(0xFFFFFFFF); // Error indicator
            }
        }
        
        Ok(())
    }
    
    fn read_leb128_u32(&self, pc: &mut usize) -> Result<u32, ExecutionError> {
        let mut result = 0u32;
        let mut shift = 0;
        
        loop {
            if *pc >= self.wasm_bytes.len() {
                return Err(ExecutionError::UnexpectedEnd);
            }
            
            let byte = self.wasm_bytes[*pc];
            *pc += 1;
            
            result |= ((byte & 0x7F) as u32) << shift;
            
            if (byte & 0x80) == 0 {
                break;
            }
            
            shift += 7;
            if shift >= 32 {
                return Err(ExecutionError::InvalidLeb128);
            }
        }
        
        Ok(result)
    }
    
    fn read_leb128_u64(&self, pc: &mut usize) -> Result<u64, ExecutionError> {
        let mut result = 0u64;
        let mut shift = 0;
        
        loop {
            if *pc >= self.wasm_bytes.len() {
                return Err(ExecutionError::UnexpectedEnd);
            }
            
            let byte = self.wasm_bytes[*pc];
            *pc += 1;
            
            result |= ((byte & 0x7F) as u64) << shift;
            
            if (byte & 0x80) == 0 {
                break;
            }
            
            shift += 7;
            if shift >= 64 {
                return Err(ExecutionError::InvalidLeb128);
            }
        }
        
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionError {
    InvalidWasm,
    UnexpectedEnd,
    UnknownOpcode(u8),
    Unreachable,
    DivisionByZero,
    InvalidLeb128,
    InstructionLimitExceeded,
    MemoryLimitExceeded,
    TimeLimitExceeded,
    HostcallFailed,
}

impl core::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ExecutionError::InvalidWasm => write!(f, "Invalid WebAssembly module"),
            ExecutionError::UnexpectedEnd => write!(f, "Unexpected end of module"),
            ExecutionError::UnknownOpcode(opcode) => write!(f, "Unknown opcode: 0x{:02X}", opcode),
            ExecutionError::Unreachable => write!(f, "Unreachable instruction executed"),
            ExecutionError::DivisionByZero => write!(f, "Division by zero"),
            ExecutionError::InvalidLeb128 => write!(f, "Invalid LEB128 encoding"),
            ExecutionError::InstructionLimitExceeded => write!(f, "Instruction limit exceeded"),
            ExecutionError::MemoryLimitExceeded => write!(f, "Memory limit exceeded"),
            ExecutionError::TimeLimitExceeded => write!(f, "Time limit exceeded"),
            ExecutionError::HostcallFailed => write!(f, "Hostcall failed"),
        }
    }
}
