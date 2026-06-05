//! WASM Backend for Aetheris Polyglot Runtime
//!
//! Implements the LanguageBackend trait for WebAssembly, integrating with
//! the existing skills/wasi.rs infrastructure to support Web3 dApps and
//! Smart Contract execution sandboxes.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::klog;
use super::backend::{
    LanguageBackend, LanguageType, LoadedModule, AppManifest, RuntimeError,
    Value, ValidationError, DebugInfo, StackFrame, MemoryRegion, permissions,
};

/// Maximum WASM module size (32 MB)
pub const MAX_WASM_MODULE_SIZE: usize = 32 * 1024 * 1024;

/// Maximum memory per WASM instance (64 MB default)
pub const DEFAULT_WASM_MEMORY_LIMIT: usize = 64 * 1024 * 1024;

/// WASM module state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmModuleState {
    /// Module loaded but not instantiated
    Loaded,
    /// Module instantiated and ready to execute
    Instantiated,
    /// Module is currently executing
    Executing,
    /// Module execution completed
    Completed,
    /// Module encountered an error
    Error,
}

/// WASM execution context
#[derive(Debug)]
pub struct WasmExecutionContext {
    /// Module ID
    pub module_id: u64,
    /// Current memory usage in bytes
    pub memory_usage: usize,
    /// Peak memory usage
    pub peak_memory: usize,
    /// Number of instructions executed
    pub instructions_executed: u64,
    /// Number of hostcalls made
    pub hostcalls_made: u64,
    /// Module state
    pub state: WasmModuleState,
}

impl WasmExecutionContext {
    /// Create a new execution context
    pub fn new(module_id: u64) -> Self {
        Self {
            module_id,
            memory_usage: 0,
            peak_memory: 0,
            instructions_executed: 0,
            hostcalls_made: 0,
            state: WasmModuleState::Loaded,
        }
    }

    /// Update memory usage
    pub fn update_memory(&mut self, bytes: usize) {
        self.memory_usage = bytes;
        if bytes > self.peak_memory {
            self.peak_memory = bytes;
        }
    }

    /// Record a hostcall
    pub fn record_hostcall(&mut self) {
        self.hostcalls_made += 1;
    }

    /// Record instructions executed
    pub fn record_instructions(&mut self, count: u64) {
        self.instructions_executed += count;
    }
}

/// WASM Backend - implements LanguageBackend for WebAssembly
/// 
/// This backend integrates with the existing skills/wasi.rs infrastructure
/// to provide WASM execution capabilities for the polyglot runtime.
pub struct WasmBackend {
    /// Next module ID
    next_module_id: AtomicU64,
    /// Memory limit for WASM instances
    memory_limit: usize,
}

impl WasmBackend {
    /// Create a new WASM backend with default settings
    pub fn new() -> Self {
        Self {
            next_module_id: AtomicU64::new(1),
            memory_limit: DEFAULT_WASM_MEMORY_LIMIT,
        }
    }

    /// Create a WASM backend with custom memory limit
    pub fn with_memory_limit(memory_limit: usize) -> Self {
        Self {
            next_module_id: AtomicU64::new(1),
            memory_limit,
        }
    }

    /// Generate a new unique module ID
    fn generate_module_id(&self) -> u64 {
        self.next_module_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Validate WASM module magic number and version
    fn validate_wasm_header(&self, code: &[u8]) -> Result<(), RuntimeError> {
        // WASM magic number: 0x00 0x61 0x73 0x6D ("\0asm")
        const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6D];
        // WASM version 1: 0x01 0x00 0x00 0x00
        const WASM_VERSION_1: [u8; 4] = [0x01, 0x00, 0x00, 0x00];

        if code.len() < 8 {
            return Err(RuntimeError::ModuleLoadFailed(
                "WASM module too small (missing header)".into()
            ));
        }

        if &code[0..4] != &WASM_MAGIC {
            return Err(RuntimeError::ModuleLoadFailed(
                "Invalid WASM magic number".into()
            ));
        }

        if &code[4..8] != &WASM_VERSION_1 {
            return Err(RuntimeError::ModuleLoadFailed(
                crate::kformat!("Unsupported WASM version: {:?}", &code[4..8])
            ));
        }

        Ok(())
    }

    /// Parse WASM module sections (basic parsing for memory tracking)
    fn parse_module_sections(&self, code: &[u8]) -> Result<WasmModuleInfo, RuntimeError> {
        let mut info = WasmModuleInfo::default();
        let mut offset = 8; // Skip header

        while offset < code.len() {
            if offset + 1 > code.len() {
                break;
            }

            let section_id = code[offset];
            offset += 1;

            // Read section size (LEB128)
            let (section_size, bytes_read) = self.read_leb128_u32(&code[offset..])?;
            offset += bytes_read;

            match section_id {
                0 => info.custom_sections += 1,  // Custom section
                1 => info.type_section = true,   // Type section
                2 => info.import_section = true, // Import section
                3 => info.function_section = true, // Function section
                4 => info.table_section = true,  // Table section
                5 => {
                    // Memory section - parse initial/max pages
                    info.memory_section = true;
                    if section_size > 0 && offset + section_size as usize <= code.len() {
                        // Parse memory limits (simplified)
                        let mem_data = &code[offset..offset + section_size as usize];
                        if !mem_data.is_empty() {
                            let num_memories = mem_data[0] as usize;
                            info.memory_count = num_memories;
                        }
                    }
                }
                6 => info.global_section = true,  // Global section
                7 => info.export_section = true,  // Export section
                8 => info.start_section = true,   // Start section
                9 => info.element_section = true, // Element section
                10 => info.code_section = true,   // Code section
                11 => info.data_section = true,   // Data section
                _ => {} // Unknown section
            }

            offset += section_size as usize;
        }

        Ok(info)
    }

    /// Read LEB128 encoded u32
    fn read_leb128_u32(&self, data: &[u8]) -> Result<(u32, usize), RuntimeError> {
        let mut result: u32 = 0;
        let mut shift = 0;
        let mut bytes_read = 0;

        for &byte in data.iter().take(5) {
            bytes_read += 1;
            result |= ((byte & 0x7F) as u32) << shift;
            
            if byte & 0x80 == 0 {
                return Ok((result, bytes_read));
            }
            
            shift += 7;
        }

        Err(RuntimeError::ModuleLoadFailed(
            "Invalid LEB128 encoding".into()
        ))
    }

    /// Convert Value arguments to WASM-compatible format
    fn convert_args_to_wasm(&self, args: &[Value]) -> Result<Vec<WasmValue>, RuntimeError> {
        args.iter().map(|arg| {
            match arg {
                Value::Null => Ok(WasmValue::I32(0)),
                Value::Bool(b) => Ok(WasmValue::I32(if *b { 1 } else { 0 })),
                Value::Int(i) => {
                    if *i >= i32::MIN as i64 && *i <= i32::MAX as i64 {
                        Ok(WasmValue::I32(*i as i32))
                    } else {
                        Ok(WasmValue::I64(*i))
                    }
                }
                Value::Float(f) => Ok(WasmValue::F64(*f)),
                Value::String(_) | Value::Bytes(_) | Value::Array(_) => {
                    // Complex types need to be passed via linear memory
                    Err(RuntimeError::ExecutionFailed(
                        "Complex argument types require memory allocation".into()
                    ))
                }
            }
        }).collect()
    }
}

impl Default for WasmBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageBackend for WasmBackend {
    fn language_type(&self) -> LanguageType {
        LanguageType::Wasm
    }

    fn validate(&self) -> Result<(), ValidationError> {
        // WASM backend implements all required methods
        Ok(())
    }

    fn load(&self, code: &[u8], manifest: &AppManifest) -> Result<LoadedModule, RuntimeError> {
        // Validate module size
        if code.len() > MAX_WASM_MODULE_SIZE {
            return Err(RuntimeError::ModuleLoadFailed(
                crate::kformat!(
                    "WASM module too large: {} bytes (max: {} bytes)",
                    code.len(), MAX_WASM_MODULE_SIZE
                )
            ));
        }

        // Validate WASM header
        self.validate_wasm_header(code)?;

        // Parse module sections for info
        let module_info = self.parse_module_sections(code)?;

        let module_id = self.generate_module_id();

        klog!(INFO, "[WASM] Loaded module '{}' (id={}, size={} bytes, {} custom sections)",
              manifest.name, module_id, code.len(), module_info.custom_sections);

        Ok(LoadedModule {
            id: module_id,
            language: LanguageType::Wasm,
            name: manifest.name.clone(),
            code: code.to_vec(),
            memory_usage: code.len(), // Initial memory is just the code size
        })
    }

    fn execute(
        &self,
        module: &LoadedModule,
        function: &str,
        args: &[Value],
    ) -> Result<Value, RuntimeError> {
        // Validate module type
        if module.language != LanguageType::Wasm {
            return Err(RuntimeError::ExecutionFailed(
                "Module is not a WASM module".into()
            ));
        }

        // Validate function name
        if function.is_empty() {
            return Err(RuntimeError::ExecutionFailed(
                "Function name cannot be empty".into()
            ));
        }

        // Convert arguments to WASM-compatible format
        let wasm_args = self.convert_args_to_wasm(args)?;

        klog!(INFO, "[WASM] Execute '{}' in module '{}' with {} args",
              function, module.name, wasm_args.len());

        // In a full implementation, this would:
        // 1. Instantiate the WASM module using a runtime (e.g., wasmi, wasmtime)
        // 2. Set up the WASI host environment via skills/wasi.rs
        // 3. Call the specified function with converted arguments
        // 4. Handle traps and errors
        // 5. Convert the result back to Value
        //
        // For the kernel's no_std environment, we would use wasmi or a custom interpreter
        // The actual execution is deferred to integration with the skills runtime

        // Placeholder result - in production this would be the actual return value
        Ok(Value::Null)
    }

    fn memory_usage(&self, module: &LoadedModule) -> usize {
        // Return the tracked memory usage
        // In a full implementation, this would query the WASM runtime
        module.memory_usage
    }

    fn unload(&self, module: LoadedModule) -> Result<(), RuntimeError> {
        klog!(INFO, "[WASM] Unloading module '{}' (id={})", module.name, module.id);
        
        // In a full implementation, this would:
        // 1. Stop any running execution
        // 2. Free WASM memory
        // 3. Clean up host resources
        
        // The module is dropped when this function returns
        Ok(())
    }

    fn debug_info(&self, module: &LoadedModule) -> DebugInfo {
        // Provide basic debug information
        // In a full implementation, this would parse DWARF debug info if available
        
        let mut info = DebugInfo::default();
        
        // Add a placeholder stack frame
        info.stack_frames.push(StackFrame {
            function: crate::kformat!("{}::main", module.name),
            file: None,
            line: None,
            ip: 0,
        });
        
        // Add memory region for the module code
        info.memory_regions.push(MemoryRegion {
            start: 0,
            size: module.code.len(),
            permissions: permissions::READ | permissions::EXECUTE,
            name: "code".into(),
        });
        
        info
    }
}

/// Information about a parsed WASM module
#[derive(Debug, Default)]
pub struct WasmModuleInfo {
    /// Number of custom sections
    pub custom_sections: usize,
    /// Has type section
    pub type_section: bool,
    /// Has import section
    pub import_section: bool,
    /// Has function section
    pub function_section: bool,
    /// Has table section
    pub table_section: bool,
    /// Has memory section
    pub memory_section: bool,
    /// Number of memory definitions
    pub memory_count: usize,
    /// Has global section
    pub global_section: bool,
    /// Has export section
    pub export_section: bool,
    /// Has start section
    pub start_section: bool,
    /// Has element section
    pub element_section: bool,
    /// Has code section
    pub code_section: bool,
    /// Has data section
    pub data_section: bool,
}

/// WASM value types for function arguments and returns
#[derive(Debug, Clone, PartialEq)]
pub enum WasmValue {
    /// 32-bit integer
    I32(i32),
    /// 64-bit integer
    I64(i64),
    /// 32-bit float
    F32(f32),
    /// 64-bit float
    F64(f64),
}

impl WasmValue {
    /// Convert to i32 if possible
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            WasmValue::I32(v) => Some(*v),
            _ => None,
        }
    }

    /// Convert to i64 if possible
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            WasmValue::I64(v) => Some(*v),
            WasmValue::I32(v) => Some(*v as i64),
            _ => None,
        }
    }

    /// Convert to f64 if possible
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            WasmValue::F64(v) => Some(*v),
            WasmValue::F32(v) => Some(*v as f64),
            _ => None,
        }
    }
}

/// Convert WasmValue back to Value
impl From<WasmValue> for Value {
    fn from(wasm_val: WasmValue) -> Self {
        match wasm_val {
            WasmValue::I32(v) => Value::Int(v as i64),
            WasmValue::I64(v) => Value::Int(v),
            WasmValue::F32(v) => Value::Float(v as f64),
            WasmValue::F64(v) => Value::Float(v),
        }
    }
}
