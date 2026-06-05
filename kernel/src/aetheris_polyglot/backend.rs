//! Language Backend Trait and Types
//!
//! Defines the interface that all language runtimes must implement to integrate
//! with the Aetheris Polyglot Runtime.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use core::fmt;

/// Supported language types for the polyglot runtime
/// 
/// # The "Avengers" of Operating Systems
/// 
/// Each language brings its superpower to the microkernel:
/// 
/// | Language | Codename     | Superpower                              | Domain                        |
/// |----------|--------------|----------------------------------------|-------------------------------|
/// | Rust     | Sentinel     | Memory Safety + Zero-Cost Abstractions | Kernel Core, Security, Crypto |
/// | Go       | Coordinator  | Goroutines + GC + Network Stack        | Services, Networking, FS      |
/// | C++      | Speedster    | Raw Performance + GPU/Hardware         | Graphics, AI, Audio, Drivers  |
/// | C#       | Architect    | Managed Runtime + Rapid Dev            | App Runtime, UI, Plugins      |
/// | WASM     | Shapeshifter | Sandboxing + Portability               | Extensions, Portable Drivers  |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum LanguageType {
    /// WebAssembly - "Shapeshifter": Sandboxing + Portability
    /// Domain: Untrusted code, portable drivers, browser extensions, isolation
    Wasm = 0,
    /// Python interpreter (scripting support)
    Python = 1,
    /// JavaScript/V8 runtime (scripting support)
    JavaScript = 2,
    /// Rust - "Sentinel": Memory Safety + Zero-Cost Abstractions
    /// Domain: Kernel core, security subsystems, cryptography, capability management
    Rust = 3,
    /// Go - "Coordinator": Goroutines + Garbage Collection + Network Stack
    /// Domain: System services, networking, filesystem, service orchestration
    Go = 4,
    /// C++ - "Speedster": Raw Performance + GPU/Hardware Access
    /// Domain: Graphics/compositor, AI inference, physics, audio processing
    Cpp = 5,
    /// C# - "Architect": Managed Runtime + Rapid Development
    /// Domain: Application runtime, UI framework, plugin system, scripting
    CSharp = 6,
}

impl LanguageType {
    /// Get the language type from a string identifier
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "wasm" | "webassembly" => Some(LanguageType::Wasm),
            "python" | "py" => Some(LanguageType::Python),
            "javascript" | "js" => Some(LanguageType::JavaScript),
            "rust" | "rs" => Some(LanguageType::Rust),
            "go" | "golang" => Some(LanguageType::Go),
            "cpp" | "c++" | "cxx" => Some(LanguageType::Cpp),
            "csharp" | "c#" | "cs" => Some(LanguageType::CSharp),
            _ => None,
        }
    }

    /// Get the string identifier for this language type
    pub fn as_str(&self) -> &'static str {
        match self {
            LanguageType::Wasm => "wasm",
            LanguageType::Python => "python",
            LanguageType::JavaScript => "javascript",
            LanguageType::Rust => "rust",
            LanguageType::Go => "go",
            LanguageType::Cpp => "cpp",
            LanguageType::CSharp => "csharp",
        }
    }
    
    /// Get the hero codename for this language
    pub fn codename(&self) -> &'static str {
        match self {
            LanguageType::Rust => "Sentinel",
            LanguageType::Go => "Coordinator",
            LanguageType::Cpp => "Speedster",
            LanguageType::CSharp => "Architect",
            LanguageType::Wasm => "Shapeshifter",
            LanguageType::Python => "Scribe",
            LanguageType::JavaScript => "Weaver",
        }
    }
    
    /// Get the superpower description for this language
    pub fn superpower(&self) -> &'static str {
        match self {
            LanguageType::Rust => "Memory Safety + Zero-Cost Abstractions",
            LanguageType::Go => "Goroutines + GC + Network Stack",
            LanguageType::Cpp => "Raw Performance + GPU/Hardware",
            LanguageType::CSharp => "Managed Runtime + Rapid Dev",
            LanguageType::Wasm => "Sandboxing + Portability",
            LanguageType::Python => "Rapid Prototyping + ML",
            LanguageType::JavaScript => "Event Loop + Web Integration",
        }
    }
    
    /// Get the primary domain for this language
    pub fn domain(&self) -> &'static str {
        match self {
            LanguageType::Rust => "Kernel Core, Security, Crypto",
            LanguageType::Go => "Services, Networking, Orchestration",
            LanguageType::Cpp => "Graphics, AI Inference, Drivers",
            LanguageType::CSharp => "App Runtime, UI, Plugins",
            LanguageType::Wasm => "Sandboxed Extensions, Portable Drivers",
            LanguageType::Python => "Scripting, ML Pipelines",
            LanguageType::JavaScript => "UI Scripting, Web Bridges",
        }
    }
    
    /// Check if this language can handle a specific task type
    pub fn can_handle(&self, task: TaskType) -> bool {
        match (self, task) {
            // Rust handles security-critical and kernel tasks
            (LanguageType::Rust, TaskType::Security) => true,
            (LanguageType::Rust, TaskType::Memory) => true,
            (LanguageType::Rust, TaskType::Crypto) => true,
            (LanguageType::Rust, TaskType::Kernel) => true,
            (LanguageType::Rust, TaskType::Capability) => true,
            
            // Go handles services and networking
            (LanguageType::Go, TaskType::Network) => true,
            (LanguageType::Go, TaskType::FileSystem) => true,
            (LanguageType::Go, TaskType::Service) => true,
            (LanguageType::Go, TaskType::Orchestration) => true,
            
            // C++ handles performance-critical tasks
            (LanguageType::Cpp, TaskType::Graphics) => true,
            (LanguageType::Cpp, TaskType::AiInference) => true,
            (LanguageType::Cpp, TaskType::Audio) => true,
            (LanguageType::Cpp, TaskType::Physics) => true,
            (LanguageType::Cpp, TaskType::Driver) => true,
            
            // C# handles application-level tasks
            (LanguageType::CSharp, TaskType::AppRuntime) => true,
            (LanguageType::CSharp, TaskType::UserInterface) => true,
            (LanguageType::CSharp, TaskType::Plugin) => true,
            (LanguageType::CSharp, TaskType::Scripting) => true,
            
            // WASM handles sandboxed/portable tasks
            (LanguageType::Wasm, TaskType::Sandbox) => true,
            (LanguageType::Wasm, TaskType::Extension) => true,
            (LanguageType::Wasm, TaskType::PortableDriver) => true,
            (LanguageType::Wasm, TaskType::Untrusted) => true,
            
            // Python/JS for scripting
            (LanguageType::Python, TaskType::Scripting) => true,
            (LanguageType::JavaScript, TaskType::Scripting) => true,
            
            _ => false,
        }
    }
    
    /// Get the best language for a task type
    pub fn best_for(task: TaskType) -> Self {
        task.best_language()
    }
    
    /// Get all "Avengers" languages (core polyglot heroes)
    pub fn avengers() -> [LanguageType; 5] {
        [
            LanguageType::Rust,
            LanguageType::Go,
            LanguageType::Cpp,
            LanguageType::CSharp,
            LanguageType::Wasm,
        ]
    }
}

/// Task types that language backends can handle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskType {
    // Rust domain (Ring 0)
    Security,
    Memory,
    Crypto,
    Kernel,
    Capability,
    
    // Go domain (Ring 3 Services)
    Network,
    FileSystem,
    Service,
    Orchestration,
    
    // C++ domain (Ring 3 Performance)
    Graphics,
    AiInference,
    Audio,
    Physics,
    Driver,
    
    // C# domain (Ring 3 Apps)
    AppRuntime,
    UserInterface,
    Plugin,
    Scripting,
    
    // WASM domain (Sandboxed)
    Sandbox,
    Extension,
    PortableDriver,
    Untrusted,
}

impl TaskType {
    /// Get the best language for this task type
    pub fn best_language(&self) -> LanguageType {
        match self {
            TaskType::Security | TaskType::Memory | TaskType::Crypto | 
            TaskType::Kernel | TaskType::Capability => LanguageType::Rust,
            
            TaskType::Network | TaskType::FileSystem | 
            TaskType::Service | TaskType::Orchestration => LanguageType::Go,
            
            TaskType::Graphics | TaskType::AiInference | 
            TaskType::Audio | TaskType::Physics | TaskType::Driver => LanguageType::Cpp,
            
            TaskType::AppRuntime | TaskType::UserInterface | 
            TaskType::Plugin | TaskType::Scripting => LanguageType::CSharp,
            
            TaskType::Sandbox | TaskType::Extension | 
            TaskType::PortableDriver | TaskType::Untrusted => LanguageType::Wasm,
        }
    }
}

impl fmt::Display for LanguageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A loaded module ready for execution
#[derive(Debug)]
pub struct LoadedModule {
    /// Unique module identifier
    pub id: u64,
    /// Language type of this module
    pub language: LanguageType,
    /// Module name
    pub name: String,
    /// Module bytecode or compiled code
    pub code: Vec<u8>,
    /// Current memory usage in bytes
    pub memory_usage: usize,
}

/// Debug information for a running application
#[derive(Debug, Clone, Default)]
pub struct DebugInfo {
    /// Stack frames
    pub stack_frames: Vec<StackFrame>,
    /// Memory regions
    pub memory_regions: Vec<MemoryRegion>,
}

/// Stack frame information
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Function name
    pub function: String,
    /// Source file (if available)
    pub file: Option<String>,
    /// Line number (if available)
    pub line: Option<u32>,
    /// Instruction pointer
    pub ip: u64,
}

/// Memory region information
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    /// Start address
    pub start: u64,
    /// Size in bytes
    pub size: usize,
    /// Permissions (read/write/execute flags)
    pub permissions: u8,
    /// Region name
    pub name: String,
}

/// Memory region permission flags
pub mod permissions {
    pub const READ: u8 = 0b001;
    pub const WRITE: u8 = 0b010;
    pub const EXECUTE: u8 = 0b100;
}

/// Validation error for backend registration
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Missing required methods
    pub missing_methods: Vec<String>,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Backend validation failed: missing methods: {:?}", self.missing_methods)
    }
}

/// Runtime error during execution
#[derive(Debug, Clone)]
pub enum RuntimeError {
    /// Language backend not found
    BackendNotFound(LanguageType),
    /// Backend validation failed
    BackendValidationFailed {
        language: LanguageType,
        missing_methods: Vec<String>,
    },
    /// Sandbox creation failed
    SandboxCreationFailed(String),
    /// Resource limit exceeded
    ResourceLimitExceeded {
        resource: ResourceType,
        limit: u64,
        requested: u64,
    },
    /// Capability verification failed
    CapabilityDenied {
        operation: String,
        reason: String,
    },
    /// Serialization error
    SerializationError(String),
    /// Deserialization error
    DeserializationError {
        offset: usize,
        reason: String,
    },
    /// Backend crashed
    BackendCrashed {
        language: LanguageType,
        reason: String,
    },
    /// Syscall failed
    SyscallFailed {
        syscall: String,
        error_code: i32,
    },
    /// Module load failed
    ModuleLoadFailed(String),
    /// Execution failed
    ExecutionFailed(String),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::BackendNotFound(lang) => {
                write!(f, "Backend not found for language: {}", lang)
            }
            RuntimeError::BackendValidationFailed { language, missing_methods } => {
                write!(f, "Backend validation failed for {}: missing {:?}", language, missing_methods)
            }
            RuntimeError::SandboxCreationFailed(reason) => {
                write!(f, "Sandbox creation failed: {}", reason)
            }
            RuntimeError::ResourceLimitExceeded { resource, limit, requested } => {
                write!(f, "Resource limit exceeded: {:?} limit={}, requested={}", resource, limit, requested)
            }
            RuntimeError::CapabilityDenied { operation, reason } => {
                write!(f, "Capability denied for {}: {}", operation, reason)
            }
            RuntimeError::SerializationError(reason) => {
                write!(f, "Serialization error: {}", reason)
            }
            RuntimeError::DeserializationError { offset, reason } => {
                write!(f, "Deserialization error at offset {}: {}", offset, reason)
            }
            RuntimeError::BackendCrashed { language, reason } => {
                write!(f, "Backend {} crashed: {}", language, reason)
            }
            RuntimeError::SyscallFailed { syscall, error_code } => {
                write!(f, "Syscall {} failed with error code {}", syscall, error_code)
            }
            RuntimeError::ModuleLoadFailed(reason) => {
                write!(f, "Module load failed: {}", reason)
            }
            RuntimeError::ExecutionFailed(reason) => {
                write!(f, "Execution failed: {}", reason)
            }
        }
    }
}

/// Resource types for limit errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    Memory,
    CpuTime,
    FileDescriptors,
    NetworkConnections,
}

/// Value type for cross-language communication
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    Array(Vec<Value>),
}

/// Application manifest describing an application's requirements
#[derive(Debug, Clone)]
pub struct AppManifest {
    /// Application name
    pub name: String,
    /// Application version
    pub version: String,
    /// Language type
    pub language: LanguageType,
    /// Entry point function
    pub entry_point: String,
    /// Resource requirements
    pub resources: ResourceRequirements,
}

/// Resource requirements from manifest
#[derive(Debug, Clone, Default)]
pub struct ResourceRequirements {
    /// Minimum memory in bytes
    pub min_memory_bytes: usize,
    /// Maximum memory in bytes
    pub max_memory_bytes: usize,
    /// Expected CPU usage (0.0-1.0)
    pub cpu_fraction: f32,
}

/// Language backend trait - implemented by each language runtime
pub trait LanguageBackend: Send + Sync {
    /// Get the language type this backend supports
    fn language_type(&self) -> LanguageType;

    /// Validate that the backend implements all required features
    fn validate(&self) -> Result<(), ValidationError>;

    /// Load application code into the runtime
    fn load(&self, code: &[u8], manifest: &AppManifest) -> Result<LoadedModule, RuntimeError>;

    /// Execute a function in the loaded module
    fn execute(&self, module: &LoadedModule, function: &str, args: &[Value]) -> Result<Value, RuntimeError>;

    /// Get memory usage of a loaded module
    fn memory_usage(&self, module: &LoadedModule) -> usize;

    /// Cleanup resources for a module
    fn unload(&self, module: LoadedModule) -> Result<(), RuntimeError>;

    /// Get backend-specific debug information
    fn debug_info(&self, module: &LoadedModule) -> DebugInfo;
}
