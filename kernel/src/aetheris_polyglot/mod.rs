//! Aetheris Polyglot Runtime Module
//!
//! This module provides a unified runtime abstraction layer that enables execution
//! of applications written in multiple programming languages. It extends the existing
//! WASM-based skills runtime to support additional language runtimes while maintaining
//! security guarantees provided by the capability-based security model.
//!
//! # Architecture
//!
//! The polyglot runtime follows a plugin-based model where language-specific backends
//! implement a common interface, allowing new languages to be added without kernel
//! modifications.
//!
//! # Components
//!
//! - `manager`: Central coordinator for all language runtimes
//! - `backend`: Language backend trait and implementations
//! - `sandbox`: Isolated execution environments with resource limits
//! - `bridge`: Interface layer translating between language APIs and kernel syscalls
//! - `registry`: Backend discovery and registration

use crate::{kprintln, klog};
use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;

pub mod backend;
pub mod bridge;
pub mod intent_adapter;
pub mod manager;
pub mod orchestrator;
pub mod registry;
pub mod sandbox;
pub mod security_adapter;
pub mod wasm_backend;

#[cfg(any(test, feature = "debug"))]
pub mod tests;

// Re-export key types for external use
pub use backend::{LanguageBackend, LanguageType, TaskType};
pub use bridge::{PolyglotBridge, AetherisOp, Message, Destination, Syscall, SyscallResult};
pub use intent_adapter::{IntentBusAdapter, AiMessageType, AiInvokeRequest, AiQueryRequest};
pub use manager::PolyglotRuntimeManager;
pub use orchestrator::{MicrokernelOrchestrator, HeroMessage, MessagePriority, TaskRequest, channels, demo_avengers};
pub use registry::BackendRegistry;
pub use sandbox::{ExecutionSandbox, ResourceLimits, ResourceUsage, SandboxState};
pub use security_adapter::{SecurityAdapter, init_security_adapter, get_security_adapter};
pub use wasm_backend::WasmBackend;

use spin::Once;

/// Global runtime manager instance (lazily initialized)
static RUNTIME_MANAGER: Once<Mutex<PolyglotRuntimeManager>> = Once::new();

/// Initialize the polyglot runtime subsystem
pub fn init() {
    kprintln!("╔══════════════════════════════════════════════════════════════╗");
    kprintln!("║     🦸 AVENGERS MICROKERNEL - ASSEMBLING THE TEAM 🦸        ║");
    kprintln!("╠══════════════════════════════════════════════════════════════╣");
    kprintln!("║  Rust    [Sentinel]     Memory Safety + Zero-Cost           ║");
    kprintln!("║  Go      [Coordinator]  Goroutines + Network Stack          ║");
    kprintln!("║  C++     [Speedster]    Raw Performance + GPU               ║");
    kprintln!("║  C#      [Architect]    Managed Runtime + UI                ║");
    kprintln!("║  WASM    [Shapeshifter] Sandboxing + Portability            ║");
    kprintln!("╚══════════════════════════════════════════════════════════════╝");
    
    // Initialize the runtime manager
    RUNTIME_MANAGER.call_once(|| Mutex::new(PolyglotRuntimeManager::new()));
    
    // Initialize the microkernel orchestrator
    orchestrator::init_orchestrator();
    
    // Initialize the security adapter for kernel integration
    security_adapter::init_security_adapter();
    
    klog!(INFO, "[POLYGLOT] Avengers Microkernel initialized - all heroes ready!");

    #[cfg(feature = "debug")]
    {
        kprintln!("[POLYGLOT] Running self-verification suite...");
        let results = tests::run_all_tests();
        let mut passed = 0;
        let mut failed = 0;
        for (name, res) in results {
            match res {
                Ok(_) => {
                    kprintln!("  [ PASS ] {}", name);
                    passed += 1;
                }
                Err(e) => {
                    kprintln!("  [ FAIL ] {}: {}", name, e);
                    failed += 1;
                }
            }
        }
        kprintln!("[POLYGLOT] Verification complete: {} passed, {} failed", passed, failed);
        if failed > 0 {
            panic!("[POLYGLOT] CRITICAL: Self-verification failed, halting system!");
        }
    }
}

/// Print the Avengers status
pub fn print_avengers_status() {
    if let Some(orch) = orchestrator::get_orchestrator().read().as_ref() {
        orch.print_status();
    }
}

/// Route a task to the best hero for the job
pub fn route_task(task_type: TaskType, operation: &str, args: Vec<backend::Value>) -> u64 {
    let request = TaskRequest {
        id: 0,
        task_type,
        operation: String::from(operation),
        args,
        priority: MessagePriority::Normal,
        requester: None,
    };
    
    if let Some(orch) = orchestrator::get_orchestrator().write().as_mut() {
        orch.route_task(request)
    } else {
        0
    }
}

/// Get the best language for a task type
pub fn best_hero_for(task: TaskType) -> LanguageType {
    task.best_language()
}

/// Get the global polyglot runtime manager
/// 
/// # Panics
/// Panics if called before `init()` has been called.
pub fn get_runtime_manager() -> &'static Mutex<PolyglotRuntimeManager> {
    RUNTIME_MANAGER.get().expect("Polyglot runtime not initialized. Call init() first.")
}
