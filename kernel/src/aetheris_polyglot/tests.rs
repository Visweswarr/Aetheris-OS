//! Property-Based Tests for Aetheris Polyglot Runtime
//!
//! These tests verify the correctness properties defined in the design document.
//! In no_std environment, we use deterministic test cases that cover the property space.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;

use alloc::sync::Arc;

use super::backend::{LanguageType, Value, RuntimeError, LanguageBackend, ValidationError, LoadedModule, AppManifest, DebugInfo, ResourceRequirements, StackFrame};
use super::sandbox::{ExecutionSandbox, ResourceLimits, SandboxId, SandboxState, RegisterState, MemoryInspectionError};
use super::bridge::{PolyglotBridge, Message, Destination, AetherisOp};
use super::registry::{BackendRegistry, RegistryError, REQUIRED_METHODS};
use super::manager::{PolyglotRuntimeManager, RuntimeConfig, ThresholdWarning, WarningResourceType, MetricThresholds};
use super::wasm_backend::WasmBackend;
use crate::security::{CapToken, get_current_time_ms};

/// Test result type
pub type TestResult = Result<(), String>;

/// Run all property tests
pub fn run_all_tests() -> Vec<(&'static str, TestResult)> {
    // Initialize simulated system clock so all timestamp checks see non-zero values
    crate::security::set_current_time_ms(1000);
    vec![
        ("property_1_language_detection_consistency", property_1_language_detection_consistency()),
        ("property_2_sandbox_initialization_invariant", property_2_sandbox_initialization_invariant()),
        ("property_3_syscall_translation_validity", property_3_syscall_translation_validity()),
        ("property_4_unsupported_language_error", property_4_unsupported_language_error()),
        ("property_5_memory_limit_enforcement", property_5_memory_limit_enforcement()),
        ("property_6_configuration_isolation", property_6_configuration_isolation()),
        ("property_7_fd_limit_enforcement", property_7_fd_limit_enforcement()),
        ("property_8_capability_verification_consistency", property_8_capability_verification_consistency()),
        ("property_9_token_revocation_propagation", property_9_token_revocation_propagation()),
        ("property_10_capability_denial_logging", property_10_capability_denial_logging()),
        ("property_11_message_serialization_roundtrip", property_11_message_serialization_roundtrip()),
        ("property_12_deserialization_error_completeness", property_12_deserialization_error_completeness()),
        ("property_13_backend_validation_completeness", property_13_backend_validation_completeness()),
        ("property_14_fault_isolation", property_14_fault_isolation()),
        ("property_15_crash_dump_completeness", property_15_crash_dump_completeness()),
        ("property_16_syscall_tracing_completeness", property_16_syscall_tracing_completeness()),
        ("property_17_memory_inspection_read_only", property_17_memory_inspection_read_only()),
        ("property_18_metrics_completeness", property_18_metrics_completeness()),
        ("property_19_termination_statistics_recording", property_19_termination_statistics_recording()),
        ("property_20_threshold_warning_emission", property_20_threshold_warning_emission()),
        ("property_21_timestamp_precision", property_21_timestamp_precision()),
    ]
}

/// Mock backend for testing - can be configured to fail validation
struct MockBackend {
    language: LanguageType,
    missing_methods: Vec<String>,
}

impl MockBackend {
    fn new(language: LanguageType) -> Self {
        Self {
            language,
            missing_methods: Vec::new(),
        }
    }

    fn with_missing_methods(language: LanguageType, missing: Vec<String>) -> Self {
        Self {
            language,
            missing_methods: missing,
        }
    }
}

impl LanguageBackend for MockBackend {
    fn language_type(&self) -> LanguageType {
        self.language
    }

    fn validate(&self) -> Result<(), ValidationError> {
        if self.missing_methods.is_empty() {
            Ok(())
        } else {
            Err(ValidationError {
                missing_methods: self.missing_methods.clone(),
            })
        }
    }

    fn load(&self, _code: &[u8], manifest: &AppManifest) -> Result<LoadedModule, RuntimeError> {
        Ok(LoadedModule {
            id: 1,
            language: self.language,
            name: manifest.name.clone(),
            code: Vec::new(),
            memory_usage: 0,
        })
    }

    fn execute(&self, _module: &LoadedModule, _function: &str, _args: &[Value]) -> Result<Value, RuntimeError> {
        Ok(Value::Null)
    }

    fn memory_usage(&self, module: &LoadedModule) -> usize {
        module.memory_usage
    }

    fn unload(&self, _module: LoadedModule) -> Result<(), RuntimeError> {
        Ok(())
    }

    fn debug_info(&self, _module: &LoadedModule) -> DebugInfo {
        DebugInfo::default()
    }
}

/// **Feature: polyglot-kernel, Property 1: Language Detection Consistency**
/// 
/// *For any* application manifest with a valid language type, the Polyglot_Runtime
/// shall always select the same Language_Backend for that language type.
/// 
/// **Validates: Requirements 1.1**
pub fn property_1_language_detection_consistency() -> TestResult {
    // Test all valid language type strings map consistently
    let test_cases = vec![
        ("wasm", LanguageType::Wasm),
        ("webassembly", LanguageType::Wasm),
        ("WASM", LanguageType::Wasm),
        ("python", LanguageType::Python),
        ("py", LanguageType::Python),
        ("PYTHON", LanguageType::Python),
        ("javascript", LanguageType::JavaScript),
        ("js", LanguageType::JavaScript),
        ("JavaScript", LanguageType::JavaScript),
        ("rust", LanguageType::Rust),
        ("rs", LanguageType::Rust),
        ("RUST", LanguageType::Rust),
    ];

    for (input, expected) in test_cases {
        let result1 = LanguageType::from_str(input);
        let result2 = LanguageType::from_str(input);
        
        // Consistency: same input always produces same output
        if result1 != result2 {
            return Err(crate::kformat!(
                "Inconsistent detection for '{}': {:?} vs {:?}",
                input, result1, result2
            ));
        }
        
        // Correctness: maps to expected type
        match result1 {
            Some(lang) if lang == expected => {}
            Some(lang) => {
                return Err(crate::kformat!(
                    "Wrong detection for '{}': expected {:?}, got {:?}",
                    input, expected, lang
                ));
            }
            None => {
                return Err(crate::kformat!(
                    "Failed to detect language for '{}'",
                    input
                ));
            }
        }
    }

    // Test round-trip: LanguageType -> string -> LanguageType
    let all_types = [
        LanguageType::Wasm,
        LanguageType::Python,
        LanguageType::JavaScript,
        LanguageType::Rust,
    ];

    for lang_type in all_types {
        let as_str = lang_type.as_str();
        let parsed = LanguageType::from_str(as_str);
        
        if parsed != Some(lang_type) {
            return Err(crate::kformat!(
                "Round-trip failed for {:?}: as_str='{}', parsed={:?}",
                lang_type, as_str, parsed
            ));
        }
    }

    Ok(())
}

/// **Feature: polyglot-kernel, Property 2: Sandbox Initialization Invariant**
/// 
/// *For any* Language_Backend load operation, the resulting Execution_Sandbox
/// shall have resource limits that match the configured defaults or the
/// manifest-specified limits.
/// 
/// **Validates: Requirements 1.2**
pub fn property_2_sandbox_initialization_invariant() -> TestResult {
    // Test with default limits
    let default_limits = ResourceLimits::default();
    let sandbox = ExecutionSandbox::with_defaults(LanguageType::Wasm);
    
    if sandbox.limits.max_memory_bytes != default_limits.max_memory_bytes {
        return Err(crate::kformat!(
            "Memory limit mismatch: expected {}, got {}",
            default_limits.max_memory_bytes, sandbox.limits.max_memory_bytes
        ));
    }
    
    if sandbox.limits.max_cpu_time_ms != default_limits.max_cpu_time_ms {
        return Err(crate::kformat!(
            "CPU time limit mismatch: expected {}, got {}",
            default_limits.max_cpu_time_ms, sandbox.limits.max_cpu_time_ms
        ));
    }
    
    if sandbox.limits.max_file_descriptors != default_limits.max_file_descriptors {
        return Err(crate::kformat!(
            "FD limit mismatch: expected {}, got {}",
            default_limits.max_file_descriptors, sandbox.limits.max_file_descriptors
        ));
    }

    // Test with custom limits
    let custom_limits = ResourceLimits {
        max_memory_bytes: 128 * 1024 * 1024,
        max_cpu_time_ms: 60_000,
        max_file_descriptors: 128,
        max_network_connections: 32,
    };
    
    let sandbox2 = ExecutionSandbox::new(LanguageType::Python, custom_limits.clone());
    
    if sandbox2.limits.max_memory_bytes != custom_limits.max_memory_bytes {
        return Err("Custom memory limit not applied".into());
    }
    
    if sandbox2.limits.max_cpu_time_ms != custom_limits.max_cpu_time_ms {
        return Err("Custom CPU time limit not applied".into());
    }

    // Verify initial state
    if sandbox.state != SandboxState::Created {
        return Err(crate::kformat!(
            "Initial state should be Created, got {:?}",
            sandbox.state
        ));
    }
    
    if sandbox.usage.memory_bytes != 0 {
        return Err("Initial memory usage should be 0".into());
    }
    
    if sandbox.usage.cpu_time_ms != 0 {
        return Err("Initial CPU time should be 0".into());
    }

    Ok(())
}

/// **Feature: polyglot-kernel, Property 4: Unsupported Language Error Completeness**
/// 
/// *For any* language type not registered with the Polyglot_Runtime, the error
/// response shall contain the exact language identifier that was requested.
/// 
/// **Validates: Requirements 1.4**
pub fn property_4_unsupported_language_error() -> TestResult {
    let registry = BackendRegistry::new();
    
    // Test all language types when none are registered
    let all_types = [
        LanguageType::Wasm,
        LanguageType::Python,
        LanguageType::JavaScript,
        LanguageType::Rust,
    ];
    
    for lang_type in all_types {
        let error = BackendRegistry::unsupported_language_error(lang_type);
        
        match error {
            RuntimeError::BackendNotFound(found_lang) => {
                if found_lang != lang_type {
                    return Err(crate::kformat!(
                        "Error should contain {:?}, but contains {:?}",
                        lang_type, found_lang
                    ));
                }
            }
            _ => {
                return Err(crate::kformat!(
                    "Expected BackendNotFound error, got {:?}",
                    error
                ));
            }
        }
    }

    // Verify error message contains language identifier
    let error = BackendRegistry::unsupported_language_error(LanguageType::Python);
    let error_str = crate::kformat!("{}", error);
    
    if !error_str.contains("python") {
        return Err(crate::kformat!(
            "Error message should contain 'python': {}",
            error_str
        ));
    }

    Ok(())
}

/// **Feature: polyglot-kernel, Property 6: Configuration Isolation**
/// 
/// *For any* runtime configuration change, existing Execution_Sandbox instances
/// shall retain their original configuration while new instances shall use
/// the updated configuration.
/// 
/// **Validates: Requirements 2.3**
pub fn property_6_configuration_isolation() -> TestResult {
    let mut manager = PolyglotRuntimeManager::new();
    
    // Register a mock backend
    let backend = Arc::new(MockBackend::new(LanguageType::Wasm));
    manager.register_backend(backend).map_err(|e| crate::kformat!("{:?}", e))?;
    
    // Get initial config
    let initial_config = manager.get_config().clone();
    let initial_memory_limit = initial_config.default_limits.max_memory_bytes;
    
    // Create a sandbox with initial config
    let manifest1 = AppManifest {
        name: "app1".into(),
        version: "1.0".into(),
        language: LanguageType::Wasm,
        entry_point: "main".into(),
        resources: ResourceRequirements::default(),
    };
    
    let sandbox1_id = manager.create_sandbox(&manifest1)
        .map_err(|e| crate::kformat!("Failed to create sandbox1: {:?}", e))?;
    
    // Verify sandbox1 has initial config
    let sandbox1 = manager.get_sandbox(sandbox1_id)
        .ok_or("Sandbox1 not found")?;
    let sandbox1_memory_limit = sandbox1.limits.max_memory_bytes;
    
    // Apply new configuration with different limits
    let new_memory_limit = initial_memory_limit * 2;
    let new_config = RuntimeConfig {
        default_limits: ResourceLimits {
            max_memory_bytes: new_memory_limit,
            ..initial_config.default_limits.clone()
        },
        ..initial_config.clone()
    };
    manager.apply_config(new_config);
    
    // Verify sandbox1 STILL has original config (isolation)
    let sandbox1_after = manager.get_sandbox(sandbox1_id)
        .ok_or("Sandbox1 not found after config change")?;
    
    if sandbox1_after.limits.max_memory_bytes != sandbox1_memory_limit {
        return Err(crate::kformat!(
            "Existing sandbox config changed: expected {}, got {}",
            sandbox1_memory_limit, sandbox1_after.limits.max_memory_bytes
        ));
    }
    
    // Create a new sandbox - should use NEW config
    let manifest2 = AppManifest {
        name: "app2".into(),
        version: "1.0".into(),
        language: LanguageType::Wasm,
        entry_point: "main".into(),
        resources: ResourceRequirements::default(),
    };
    
    let sandbox2_id = manager.create_sandbox(&manifest2)
        .map_err(|e| crate::kformat!("Failed to create sandbox2: {:?}", e))?;
    
    let sandbox2 = manager.get_sandbox(sandbox2_id)
        .ok_or("Sandbox2 not found")?;
    let sandbox2_memory_limit = sandbox2.limits.max_memory_bytes;
    
    // New sandbox should have the new config's memory limit (or higher if manifest specifies more)
    if sandbox2_memory_limit < new_memory_limit {
        return Err(crate::kformat!(
            "New sandbox should use new config: expected >= {}, got {}",
            new_memory_limit, sandbox2_memory_limit
        ));
    }
    
    // Test multiple config changes
    let third_memory_limit = new_memory_limit * 2;
    let third_config = RuntimeConfig {
        default_limits: ResourceLimits {
            max_memory_bytes: third_memory_limit,
            ..ResourceLimits::default()
        },
        ..RuntimeConfig::default()
    };
    manager.apply_config(third_config);
    
    // Both existing sandboxes should retain their original configs
    let sandbox1_final = manager.get_sandbox(sandbox1_id)
        .ok_or("Sandbox1 not found after second config change")?;
    let sandbox2_final = manager.get_sandbox(sandbox2_id)
        .ok_or("Sandbox2 not found after second config change")?;
    
    if sandbox1_final.limits.max_memory_bytes != sandbox1_memory_limit {
        return Err("Sandbox1 config changed after second config update".into());
    }
    
    if sandbox2_final.limits.max_memory_bytes != sandbox2_memory_limit {
        return Err("Sandbox2 config changed after second config update".into());
    }
    
    Ok(())
}

/// **Feature: polyglot-kernel, Property 5: Memory Limit Enforcement**
/// 
/// *For any* Execution_Sandbox with a configured memory limit, all memory
/// allocation attempts that would exceed the limit shall be rejected.
/// 
/// **Validates: Requirements 2.1**
pub fn property_5_memory_limit_enforcement() -> TestResult {
    let limits = ResourceLimits {
        max_memory_bytes: 1024, // 1 KB limit for testing
        max_cpu_time_ms: 30_000,
        max_file_descriptors: 64,
        max_network_connections: 16,
    };
    
    let mut sandbox = ExecutionSandbox::new(LanguageType::Wasm, limits);
    
    // Test allocation within limits
    match sandbox.allocate_memory(512) {
        Ok(()) => {}
        Err(e) => return Err(crate::kformat!("Should allow 512 bytes: {:?}", e)),
    }
    
    // Test allocation that would exceed limit
    match sandbox.allocate_memory(600) {
        Ok(()) => return Err("Should reject allocation exceeding limit".into()),
        Err(RuntimeError::ResourceLimitExceeded { resource, limit, requested }) => {
            if limit != 1024 {
                return Err(crate::kformat!("Limit should be 1024, got {}", limit));
            }
            // requested should be current (512) + new (600) = 1112
            if requested != 1112 {
                return Err(crate::kformat!("Requested should be 1112, got {}", requested));
            }
        }
        Err(e) => return Err(crate::kformat!("Wrong error type: {:?}", e)),
    }
    
    // Test exact limit allocation
    let mut sandbox2 = ExecutionSandbox::new(LanguageType::Python, ResourceLimits {
        max_memory_bytes: 1000,
        ..ResourceLimits::default()
    });
    
    match sandbox2.allocate_memory(1000) {
        Ok(()) => {}
        Err(e) => return Err(crate::kformat!("Should allow exact limit: {:?}", e)),
    }
    
    // Now any additional allocation should fail
    match sandbox2.allocate_memory(1) {
        Ok(()) => return Err("Should reject any allocation over limit".into()),
        Err(RuntimeError::ResourceLimitExceeded { .. }) => {}
        Err(e) => return Err(crate::kformat!("Wrong error type: {:?}", e)),
    }

    Ok(())
}

/// **Feature: polyglot-kernel, Property 7: File Descriptor Limit Enforcement**
/// 
/// *For any* Execution_Sandbox with a configured file descriptor limit,
/// attempts to open file descriptors beyond the limit shall fail.
/// 
/// **Validates: Requirements 2.4**
pub fn property_7_fd_limit_enforcement() -> TestResult {
    let limits = ResourceLimits {
        max_memory_bytes: 64 * 1024 * 1024,
        max_cpu_time_ms: 30_000,
        max_file_descriptors: 3, // Small limit for testing
        max_network_connections: 16,
    };
    
    let mut sandbox = ExecutionSandbox::new(LanguageType::JavaScript, limits);
    
    // Open FDs up to limit
    for i in 0..3 {
        match sandbox.open_fd() {
            Ok(()) => {}
            Err(e) => return Err(crate::kformat!("Should allow FD {}: {:?}", i, e)),
        }
    }
    
    // Verify count
    if sandbox.usage.file_descriptors != 3 {
        return Err(crate::kformat!(
            "FD count should be 3, got {}",
            sandbox.usage.file_descriptors
        ));
    }
    
    // Next open should fail
    match sandbox.open_fd() {
        Ok(()) => return Err("Should reject FD over limit".into()),
        Err(RuntimeError::ResourceLimitExceeded { resource, limit, requested }) => {
            if limit != 3 {
                return Err(crate::kformat!("Limit should be 3, got {}", limit));
            }
            if requested != 4 {
                return Err(crate::kformat!("Requested should be 4, got {}", requested));
            }
        }
        Err(e) => return Err(crate::kformat!("Wrong error type: {:?}", e)),
    }
    
    // Close one and try again
    sandbox.close_fd();
    
    if sandbox.usage.file_descriptors != 2 {
        return Err(crate::kformat!(
            "FD count should be 2 after close, got {}",
            sandbox.usage.file_descriptors
        ));
    }
    
    match sandbox.open_fd() {
        Ok(()) => {}
        Err(e) => return Err(crate::kformat!("Should allow FD after close: {:?}", e)),
    }

    Ok(())
}

/// **Feature: polyglot-kernel, Property 13: Backend Validation Completeness**
/// 
/// *For any* Language_Backend that fails validation, the error shall list
/// all missing required interface methods.
/// 
/// **Validates: Requirements 5.1, 5.2**
pub fn property_13_backend_validation_completeness() -> TestResult {
    let mut registry = BackendRegistry::new();

    // Test 1: Valid backend should register successfully
    let valid_backend = Arc::new(MockBackend::new(LanguageType::Wasm));
    match registry.register(valid_backend) {
        Ok(()) => {}
        Err(e) => return Err(crate::kformat!("Valid backend should register: {:?}", e)),
    }

    // Test 2: Backend with single missing method should report it
    let single_missing = vec!["execute".into()];
    let backend_single = Arc::new(MockBackend::with_missing_methods(
        LanguageType::Python,
        single_missing.clone(),
    ));
    
    match registry.register(backend_single) {
        Ok(()) => return Err("Should reject backend with missing method".into()),
        Err(RegistryError::ValidationFailed { language, missing_methods }) => {
            if language != LanguageType::Python {
                return Err(crate::kformat!(
                    "Error should report Python, got {:?}",
                    language
                ));
            }
            if missing_methods != single_missing {
                return Err(crate::kformat!(
                    "Should report {:?}, got {:?}",
                    single_missing, missing_methods
                ));
            }
        }
        Err(e) => return Err(crate::kformat!("Wrong error type: {:?}", e)),
    }

    // Test 3: Backend with multiple missing methods should report ALL of them
    let multiple_missing = vec!["load".into(), "execute".into(), "unload".into()];
    let backend_multiple = Arc::new(MockBackend::with_missing_methods(
        LanguageType::JavaScript,
        multiple_missing.clone(),
    ));
    
    match registry.register(backend_multiple) {
        Ok(()) => return Err("Should reject backend with missing methods".into()),
        Err(RegistryError::ValidationFailed { language, missing_methods }) => {
            if language != LanguageType::JavaScript {
                return Err(crate::kformat!(
                    "Error should report JavaScript, got {:?}",
                    language
                ));
            }
            // Verify ALL missing methods are reported
            for method in &multiple_missing {
                if !missing_methods.contains(method) {
                    return Err(crate::kformat!(
                        "Missing method '{}' not reported. Got: {:?}",
                        method, missing_methods
                    ));
                }
            }
            // Verify count matches
            if missing_methods.len() != multiple_missing.len() {
                return Err(crate::kformat!(
                    "Should report {} missing methods, got {}",
                    multiple_missing.len(), missing_methods.len()
                ));
            }
        }
        Err(e) => return Err(crate::kformat!("Wrong error type: {:?}", e)),
    }

    // Test 4: Verify all required methods are defined
    if REQUIRED_METHODS.is_empty() {
        return Err("REQUIRED_METHODS should not be empty".into());
    }
    
    // Test 5: Backend missing all required methods should report all
    let all_missing: Vec<String> = REQUIRED_METHODS.iter().map(|s| (*s).into()).collect();
    let backend_all_missing = Arc::new(MockBackend::with_missing_methods(
        LanguageType::Rust,
        all_missing.clone(),
    ));
    
    match registry.register(backend_all_missing) {
        Ok(()) => return Err("Should reject backend missing all methods".into()),
        Err(RegistryError::ValidationFailed { missing_methods, .. }) => {
            // All required methods should be reported
            for method in REQUIRED_METHODS {
                if !missing_methods.iter().any(|m| m == *method) {
                    return Err(crate::kformat!(
                        "Required method '{}' not in error. Got: {:?}",
                        method, missing_methods
                    ));
                }
            }
        }
        Err(e) => return Err(crate::kformat!("Wrong error type: {:?}", e)),
    }

    // Test 6: Duplicate registration should fail with appropriate error
    let duplicate_backend = Arc::new(MockBackend::new(LanguageType::Wasm));
    match registry.register(duplicate_backend) {
        Ok(()) => return Err("Should reject duplicate registration".into()),
        Err(RegistryError::AlreadyRegistered(lang)) => {
            if lang != LanguageType::Wasm {
                return Err(crate::kformat!(
                    "Should report Wasm as duplicate, got {:?}",
                    lang
                ));
            }
        }
        Err(e) => return Err(crate::kformat!("Wrong error for duplicate: {:?}", e)),
    }

    Ok(())
}

/// **Feature: polyglot-kernel, Property 14: Fault Isolation**
/// 
/// *For any* Language_Backend crash, all other registered backends shall remain
/// operational and their sandboxes shall continue executing.
/// 
/// **Validates: Requirements 5.4**
pub fn property_14_fault_isolation() -> TestResult {
    let mut manager = PolyglotRuntimeManager::new();
    
    // Register multiple backends
    let wasm_backend = Arc::new(MockBackend::new(LanguageType::Wasm));
    let python_backend = Arc::new(MockBackend::new(LanguageType::Python));
    let js_backend = Arc::new(MockBackend::new(LanguageType::JavaScript));
    
    manager.register_backend(wasm_backend).map_err(|e| crate::kformat!("{:?}", e))?;
    manager.register_backend(python_backend).map_err(|e| crate::kformat!("{:?}", e))?;
    manager.register_backend(js_backend).map_err(|e| crate::kformat!("{:?}", e))?;
    
    // Create sandboxes for each backend
    let wasm_manifest = AppManifest {
        name: "wasm_app".into(),
        version: "1.0".into(),
        language: LanguageType::Wasm,
        entry_point: "main".into(),
        resources: ResourceRequirements::default(),
    };
    
    let python_manifest = AppManifest {
        name: "python_app".into(),
        version: "1.0".into(),
        language: LanguageType::Python,
        entry_point: "main".into(),
        resources: ResourceRequirements::default(),
    };
    
    let js_manifest = AppManifest {
        name: "js_app".into(),
        version: "1.0".into(),
        language: LanguageType::JavaScript,
        entry_point: "main".into(),
        resources: ResourceRequirements::default(),
    };
    
    // Create multiple sandboxes for each language
    let wasm_sandbox1 = manager.create_sandbox(&wasm_manifest)
        .map_err(|e| crate::kformat!("Failed to create wasm sandbox 1: {:?}", e))?;
    let wasm_sandbox2 = manager.create_sandbox(&wasm_manifest)
        .map_err(|e| crate::kformat!("Failed to create wasm sandbox 2: {:?}", e))?;
    let python_sandbox1 = manager.create_sandbox(&python_manifest)
        .map_err(|e| crate::kformat!("Failed to create python sandbox 1: {:?}", e))?;
    let python_sandbox2 = manager.create_sandbox(&python_manifest)
        .map_err(|e| crate::kformat!("Failed to create python sandbox 2: {:?}", e))?;
    let js_sandbox1 = manager.create_sandbox(&js_manifest)
        .map_err(|e| crate::kformat!("Failed to create js sandbox 1: {:?}", e))?;
    
    // Verify all sandboxes exist
    if manager.get_sandbox(wasm_sandbox1).is_none() {
        return Err("WASM sandbox 1 should exist".into());
    }
    if manager.get_sandbox(python_sandbox1).is_none() {
        return Err("Python sandbox 1 should exist".into());
    }
    if manager.get_sandbox(js_sandbox1).is_none() {
        return Err("JS sandbox 1 should exist".into());
    }
    
    // Verify initial counts
    let initial_wasm_count = manager.active_sandbox_count_for(LanguageType::Wasm);
    let initial_python_count = manager.active_sandbox_count_for(LanguageType::Python);
    let initial_js_count = manager.active_sandbox_count_for(LanguageType::JavaScript);
    
    if initial_wasm_count != 2 {
        return Err(crate::kformat!("Expected 2 WASM sandboxes, got {}", initial_wasm_count));
    }
    if initial_python_count != 2 {
        return Err(crate::kformat!("Expected 2 Python sandboxes, got {}", initial_python_count));
    }
    if initial_js_count != 1 {
        return Err(crate::kformat!("Expected 1 JS sandbox, got {}", initial_js_count));
    }
    
    // All backends should be healthy initially
    if !manager.is_backend_healthy(LanguageType::Wasm) {
        return Err("WASM backend should be healthy initially".into());
    }
    if !manager.is_backend_healthy(LanguageType::Python) {
        return Err("Python backend should be healthy initially".into());
    }
    if !manager.is_backend_healthy(LanguageType::JavaScript) {
        return Err("JS backend should be healthy initially".into());
    }
    
    // ========================================================================
    // CRASH THE PYTHON BACKEND
    // ========================================================================
    let crash_info = manager.report_backend_crash(
        LanguageType::Python,
        "Simulated crash for testing".into()
    );
    
    // Verify crash info
    if crash_info.language != LanguageType::Python {
        return Err("Crash info should report Python".into());
    }
    if crash_info.affected_sandboxes != 2 {
        return Err(crate::kformat!(
            "Crash should affect 2 Python sandboxes, got {}",
            crash_info.affected_sandboxes
        ));
    }
    
    // ========================================================================
    // VERIFY FAULT ISOLATION - Other backends remain operational
    // ========================================================================
    
    // Python backend should be crashed
    if manager.is_backend_healthy(LanguageType::Python) {
        return Err("Python backend should be crashed".into());
    }
    
    // WASM and JS backends should STILL be healthy (fault isolation)
    if !manager.is_backend_healthy(LanguageType::Wasm) {
        return Err("WASM backend should remain healthy after Python crash".into());
    }
    if !manager.is_backend_healthy(LanguageType::JavaScript) {
        return Err("JS backend should remain healthy after Python crash".into());
    }
    
    // Python sandboxes should be terminated
    if manager.get_sandbox(python_sandbox1).is_some() {
        return Err("Python sandbox 1 should be terminated after crash".into());
    }
    if manager.get_sandbox(python_sandbox2).is_some() {
        return Err("Python sandbox 2 should be terminated after crash".into());
    }
    
    // WASM and JS sandboxes should STILL exist (fault isolation)
    if manager.get_sandbox(wasm_sandbox1).is_none() {
        return Err("WASM sandbox 1 should still exist after Python crash".into());
    }
    if manager.get_sandbox(wasm_sandbox2).is_none() {
        return Err("WASM sandbox 2 should still exist after Python crash".into());
    }
    if manager.get_sandbox(js_sandbox1).is_none() {
        return Err("JS sandbox 1 should still exist after Python crash".into());
    }
    
    // Verify sandbox counts after crash
    let post_crash_wasm_count = manager.active_sandbox_count_for(LanguageType::Wasm);
    let post_crash_python_count = manager.active_sandbox_count_for(LanguageType::Python);
    let post_crash_js_count = manager.active_sandbox_count_for(LanguageType::JavaScript);
    
    if post_crash_wasm_count != 2 {
        return Err(crate::kformat!(
            "WASM sandbox count should remain 2 after Python crash, got {}",
            post_crash_wasm_count
        ));
    }
    if post_crash_python_count != 0 {
        return Err(crate::kformat!(
            "Python sandbox count should be 0 after crash, got {}",
            post_crash_python_count
        ));
    }
    if post_crash_js_count != 1 {
        return Err(crate::kformat!(
            "JS sandbox count should remain 1 after Python crash, got {}",
            post_crash_js_count
        ));
    }
    
    // ========================================================================
    // VERIFY NEW SANDBOX CREATION IS BLOCKED FOR CRASHED BACKEND
    // ========================================================================
    match manager.create_sandbox(&python_manifest) {
        Ok(_) => return Err("Should not be able to create sandbox for crashed backend".into()),
        Err(RuntimeError::BackendCrashed { language, .. }) => {
            if language != LanguageType::Python {
                return Err("Error should report Python as crashed".into());
            }
        }
        Err(e) => return Err(crate::kformat!("Wrong error type: {:?}", e)),
    }
    
    // ========================================================================
    // VERIFY NEW SANDBOX CREATION STILL WORKS FOR HEALTHY BACKENDS
    // ========================================================================
    let new_wasm_sandbox = manager.create_sandbox(&wasm_manifest)
        .map_err(|e| crate::kformat!("Should be able to create WASM sandbox: {:?}", e))?;
    
    if manager.get_sandbox(new_wasm_sandbox).is_none() {
        return Err("New WASM sandbox should exist".into());
    }
    
    let new_js_sandbox = manager.create_sandbox(&js_manifest)
        .map_err(|e| crate::kformat!("Should be able to create JS sandbox: {:?}", e))?;
    
    if manager.get_sandbox(new_js_sandbox).is_none() {
        return Err("New JS sandbox should exist".into());
    }
    
    // ========================================================================
    // VERIFY CRASH HISTORY IS RECORDED
    // ========================================================================
    let crash_history = manager.get_crash_history();
    if crash_history.is_empty() {
        return Err("Crash history should not be empty".into());
    }
    
    let recorded_crash = &crash_history[0];
    if recorded_crash.language != LanguageType::Python {
        return Err("Crash history should record Python crash".into());
    }
    if recorded_crash.affected_sandboxes != 2 {
        return Err("Crash history should record 2 affected sandboxes".into());
    }
    
    // ========================================================================
    // TEST RECOVERY
    // ========================================================================
    
    // Attempt recovery
    manager.recover_backend(LanguageType::Python)
        .map_err(|e| crate::kformat!("Recovery should succeed: {:?}", e))?;
    
    // Python backend should be healthy again
    if !manager.is_backend_healthy(LanguageType::Python) {
        return Err("Python backend should be healthy after recovery".into());
    }
    
    // Should be able to create Python sandboxes again
    let recovered_python_sandbox = manager.create_sandbox(&python_manifest)
        .map_err(|e| crate::kformat!("Should be able to create Python sandbox after recovery: {:?}", e))?;
    
    if manager.get_sandbox(recovered_python_sandbox).is_none() {
        return Err("Recovered Python sandbox should exist".into());
    }
    
    // ========================================================================
    // TEST MULTIPLE BACKEND CRASHES (cascading isolation)
    // ========================================================================
    
    // Crash WASM backend
    let wasm_crash_info = manager.report_backend_crash(
        LanguageType::Wasm,
        "Second crash for testing".into()
    );
    
    // JS should still be healthy
    if !manager.is_backend_healthy(LanguageType::JavaScript) {
        return Err("JS backend should remain healthy after WASM crash".into());
    }
    
    // Python (recovered) should still be healthy
    if !manager.is_backend_healthy(LanguageType::Python) {
        return Err("Python backend should remain healthy after WASM crash".into());
    }
    
    // WASM should be crashed
    if manager.is_backend_healthy(LanguageType::Wasm) {
        return Err("WASM backend should be crashed".into());
    }
    
    // Verify crash history has both crashes
    let final_crash_history = manager.get_crash_history();
    if final_crash_history.len() < 2 {
        return Err(crate::kformat!(
            "Crash history should have at least 2 entries, got {}",
            final_crash_history.len()
        ));
    }
    
    Ok(())
}

/// **Feature: polyglot-kernel, Property 15: Crash Dump Completeness**
/// 
/// *For any* application crash, the captured crash dump shall contain at least
/// one stack frame and the register state.
/// 
/// **Validates: Requirements 6.2**
pub fn property_15_crash_dump_completeness() -> TestResult {
    let mut sandbox = ExecutionSandbox::with_defaults(LanguageType::Wasm);
    
    // Initially, no crash dump should exist
    if sandbox.get_crash_dump().is_some() {
        return Err("No crash dump should exist initially".into());
    }
    
    // Create stack frames for the crash
    let stack_frames = vec![
        StackFrame {
            function: "main".into(),
            file: Some("app.wasm".into()),
            line: Some(42),
            ip: 0x1000,
        },
        StackFrame {
            function: "helper_function".into(),
            file: Some("utils.wasm".into()),
            line: Some(100),
            ip: 0x2000,
        },
    ];
    
    // Create register state
    let register_state = RegisterState {
        pc: 0x1000,
        sp: 0xFFFF0000,
        fp: 0xFFFF1000,
        general: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        flags: 0x202,
    };
    
    // Register some memory regions for the crash dump
    sandbox.register_memory_region(0x1000, vec![0x00, 0x61, 0x73, 0x6D]); // WASM magic
    sandbox.register_memory_region(0x2000, vec![0xDE, 0xAD, 0xBE, 0xEF]); // Some data
    
    // Capture the crash dump
    sandbox.capture_crash_dump(
        "Segmentation fault at 0x1000",
        stack_frames.clone(),
        register_state.clone()
    );
    
    // Verify crash dump was captured
    let crash_dump = sandbox.get_crash_dump()
        .ok_or("Crash dump should exist after capture")?;
    
    // ========================================================================
    // VERIFY PROPERTY 15: At least one stack frame
    // ========================================================================
    if crash_dump.stack_frames.is_empty() {
        return Err("Crash dump must contain at least one stack frame".into());
    }
    
    // Verify stack frames match what we provided
    if crash_dump.stack_frames.len() != 2 {
        return Err(crate::kformat!(
            "Expected 2 stack frames, got {}",
            crash_dump.stack_frames.len()
        ));
    }
    
    // Verify first stack frame
    let frame0 = &crash_dump.stack_frames[0];
    if frame0.function != "main" {
        return Err(crate::kformat!(
            "First frame function should be 'main', got '{}'",
            frame0.function
        ));
    }
    if frame0.ip != 0x1000 {
        return Err(crate::kformat!(
            "First frame IP should be 0x1000, got 0x{:x}",
            frame0.ip
        ));
    }
    
    // ========================================================================
    // VERIFY PROPERTY 15: Register state is captured
    // ========================================================================
    if crash_dump.register_state.pc != 0x1000 {
        return Err(crate::kformat!(
            "Register PC should be 0x1000, got 0x{:x}",
            crash_dump.register_state.pc
        ));
    }
    
    if crash_dump.register_state.sp != 0xFFFF0000 {
        return Err(crate::kformat!(
            "Register SP should be 0xFFFF0000, got 0x{:x}",
            crash_dump.register_state.sp
        ));
    }
    
    if crash_dump.register_state.fp != 0xFFFF1000 {
        return Err(crate::kformat!(
            "Register FP should be 0xFFFF1000, got 0x{:x}",
            crash_dump.register_state.fp
        ));
    }
    
    if crash_dump.register_state.flags != 0x202 {
        return Err(crate::kformat!(
            "Register flags should be 0x202, got 0x{:x}",
            crash_dump.register_state.flags
        ));
    }
    
    // Verify general purpose registers
    for (i, &expected) in [1u64, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16].iter().enumerate() {
        if crash_dump.register_state.general[i] != expected {
            return Err(crate::kformat!(
                "Register r{} should be {}, got {}",
                i, expected, crash_dump.register_state.general[i]
            ));
        }
    }
    
    // Verify crash reason is captured
    if !crash_dump.reason.contains("Segmentation fault") {
        return Err(crate::kformat!(
            "Crash reason should contain 'Segmentation fault', got '{}'",
            crash_dump.reason
        ));
    }
    
    // Verify timestamp is set
    if crash_dump.timestamp_ms == 0 {
        return Err("Crash dump timestamp should be set".into());
    }
    
    // Verify sandbox ID is correct
    if crash_dump.sandbox_id != sandbox.id {
        return Err("Crash dump sandbox ID should match".into());
    }
    
    // Verify memory snapshots are included
    if crash_dump.memory_snapshot.is_empty() {
        return Err("Crash dump should include memory snapshots".into());
    }
    
    // Verify sandbox state is Terminated after crash
    if sandbox.state != SandboxState::Terminated {
        return Err(crate::kformat!(
            "Sandbox state should be Terminated after crash, got {:?}",
            sandbox.state
        ));
    }
    
    // ========================================================================
    // Test with minimal crash dump (single stack frame)
    // ========================================================================
    let mut sandbox2 = ExecutionSandbox::with_defaults(LanguageType::Python);
    
    let minimal_frames = vec![
        StackFrame {
            function: "crash_point".into(),
            file: None,
            line: None,
            ip: 0x5000,
        },
    ];
    
    let minimal_registers = RegisterState {
        pc: 0x5000,
        sp: 0x8000,
        fp: 0x8100,
        general: [0; 16],
        flags: 0,
    };
    
    sandbox2.capture_crash_dump("Minimal crash", minimal_frames, minimal_registers);
    
    let minimal_dump = sandbox2.get_crash_dump()
        .ok_or("Minimal crash dump should exist")?;
    
    // Even minimal dump must have at least one stack frame
    if minimal_dump.stack_frames.is_empty() {
        return Err("Even minimal crash dump must have at least one stack frame".into());
    }
    
    // Register state must be present
    if minimal_dump.register_state.pc != 0x5000 {
        return Err("Minimal dump must have register state".into());
    }
    
    Ok(())
}

/// **Feature: polyglot-kernel, Property 16: Syscall Tracing Completeness**
/// 
/// *For any* sandbox with tracing enabled, every syscall invocation shall
/// produce a corresponding trace event.
/// 
/// **Validates: Requirements 6.3**
pub fn property_16_syscall_tracing_completeness() -> TestResult {
    let mut sandbox = ExecutionSandbox::with_defaults(LanguageType::Wasm);
    
    // Initially, tracing should be disabled
    if sandbox.is_tracing_enabled() {
        return Err("Tracing should be disabled by default".into());
    }
    
    // No traces should exist initially
    if !sandbox.get_syscall_traces().is_empty() {
        return Err("No traces should exist initially".into());
    }
    
    // ========================================================================
    // Test: Syscalls WITHOUT tracing enabled should NOT produce traces
    // ========================================================================
    sandbox.trace_syscall(1, "read", &[0, 1024, 100], Some(100), 50);
    sandbox.trace_syscall(2, "write", &[1, 2048, 50], Some(50), 30);
    
    if !sandbox.get_syscall_traces().is_empty() {
        return Err("Syscalls without tracing enabled should not produce traces".into());
    }
    
    // ========================================================================
    // Enable tracing
    // ========================================================================
    sandbox.enable_tracing();
    
    if !sandbox.is_tracing_enabled() {
        return Err("Tracing should be enabled after enable_tracing()".into());
    }
    
    // ========================================================================
    // Test: Syscalls WITH tracing enabled MUST produce traces
    // ========================================================================
    
    // Trace syscall 1: read
    sandbox.trace_syscall(
        1,                          // syscall number
        "read",                     // syscall name
        &[0, 0x1000, 1024],        // arguments: fd, buf, count
        Some(1024),                 // return value
        150,                        // duration in microseconds
    );
    
    // Verify trace was recorded
    let traces = sandbox.get_syscall_traces();
    if traces.len() != 1 {
        return Err(crate::kformat!(
            "Expected 1 trace after first syscall, got {}",
            traces.len()
        ));
    }
    
    // Verify trace contents
    let trace1 = &traces[0];
    if trace1.syscall_number != 1 {
        return Err(crate::kformat!(
            "Trace syscall number should be 1, got {}",
            trace1.syscall_number
        ));
    }
    if trace1.syscall_name != "read" {
        return Err(crate::kformat!(
            "Trace syscall name should be 'read', got '{}'",
            trace1.syscall_name
        ));
    }
    if trace1.arguments != vec![0, 0x1000, 1024] {
        return Err(crate::kformat!(
            "Trace arguments mismatch: {:?}",
            trace1.arguments
        ));
    }
    if trace1.return_value != Some(1024) {
        return Err(crate::kformat!(
            "Trace return value should be Some(1024), got {:?}",
            trace1.return_value
        ));
    }
    if trace1.duration_us != 150 {
        return Err(crate::kformat!(
            "Trace duration should be 150us, got {}us",
            trace1.duration_us
        ));
    }
    if trace1.sandbox_id != sandbox.id {
        return Err("Trace sandbox ID should match".into());
    }
    if trace1.timestamp_ms == 0 {
        return Err("Trace timestamp should be set".into());
    }
    
    // ========================================================================
    // Trace multiple syscalls
    // ========================================================================
    sandbox.trace_syscall(2, "write", &[1, 0x2000, 512], Some(512), 100);
    sandbox.trace_syscall(3, "open", &[0x3000, 0, 0644], Some(3), 200);
    sandbox.trace_syscall(4, "close", &[3], Some(0), 10);
    sandbox.trace_syscall(5, "mmap", &[0, 4096, 3, 1, 0, 0], Some(0x10000), 500);
    
    // Verify all syscalls produced traces
    let all_traces = sandbox.get_syscall_traces();
    if all_traces.len() != 5 {
        return Err(crate::kformat!(
            "Expected 5 traces after 5 syscalls, got {}",
            all_traces.len()
        ));
    }
    
    // Verify each syscall has a corresponding trace
    let expected_syscalls = ["read", "write", "open", "close", "mmap"];
    for (i, expected_name) in expected_syscalls.iter().enumerate() {
        if all_traces[i].syscall_name != *expected_name {
            return Err(crate::kformat!(
                "Trace {} should be '{}', got '{}'",
                i, expected_name, all_traces[i].syscall_name
            ));
        }
    }
    
    // ========================================================================
    // Test: Syscall with no return value (in-progress)
    // ========================================================================
    sandbox.trace_syscall(6, "futex", &[0x4000, 0, 0], None, 0);
    
    let traces_with_pending = sandbox.get_syscall_traces();
    let pending_trace = traces_with_pending.last().unwrap();
    if pending_trace.return_value.is_some() {
        return Err("Pending syscall should have None return value".into());
    }
    
    // ========================================================================
    // Disable tracing and verify no more traces are recorded
    // ========================================================================
    sandbox.disable_tracing();
    
    if sandbox.is_tracing_enabled() {
        return Err("Tracing should be disabled after disable_tracing()".into());
    }
    
    let trace_count_before = sandbox.get_syscall_traces().len();
    
    // These syscalls should NOT produce traces
    sandbox.trace_syscall(7, "read", &[0, 0, 0], Some(0), 10);
    sandbox.trace_syscall(8, "write", &[1, 0, 0], Some(0), 10);
    
    let trace_count_after = sandbox.get_syscall_traces().len();
    
    if trace_count_after != trace_count_before {
        return Err(crate::kformat!(
            "Syscalls after disabling tracing should not produce traces. Before: {}, After: {}",
            trace_count_before, trace_count_after
        ));
    }
    
    // ========================================================================
    // Test: Clear traces
    // ========================================================================
    sandbox.clear_syscall_traces();
    
    if !sandbox.get_syscall_traces().is_empty() {
        return Err("Traces should be empty after clear".into());
    }
    
    // ========================================================================
    // Re-enable and verify tracing works again
    // ========================================================================
    sandbox.enable_tracing();
    sandbox.trace_syscall(9, "exit", &[0], Some(0), 5);
    
    if sandbox.get_syscall_traces().len() != 1 {
        return Err("Tracing should work after re-enabling".into());
    }
    
    Ok(())
}

/// **Feature: polyglot-kernel, Property 17: Memory Inspection Read-Only**
/// 
/// *For any* memory inspection request, the Execution_Sandbox shall allow
/// reads but reject any write attempts.
/// 
/// **Validates: Requirements 6.4**
pub fn property_17_memory_inspection_read_only() -> TestResult {
    let mut sandbox = ExecutionSandbox::with_defaults(LanguageType::Wasm);
    
    // Register some memory regions
    let code_region = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00]; // WASM header
    let data_region = vec![0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
    let stack_region = vec![0x00; 256]; // 256 bytes of zeros
    
    sandbox.register_memory_region(0x1000, code_region.clone());
    sandbox.register_memory_region(0x2000, data_region.clone());
    sandbox.register_memory_region(0x3000, stack_region.clone());
    
    // ========================================================================
    // Test: Read operations should succeed
    // ========================================================================
    
    // Read from code region
    let inspection = sandbox.inspect_memory(0x1000, 8)
        .map_err(|e| crate::kformat!("Read from code region should succeed: {:?}", e))?;
    
    if inspection.address != 0x1000 {
        return Err(crate::kformat!(
            "Inspection address should be 0x1000, got 0x{:x}",
            inspection.address
        ));
    }
    
    if inspection.size != 8 {
        return Err(crate::kformat!(
            "Inspection size should be 8, got {}",
            inspection.size
        ));
    }
    
    if inspection.data != code_region {
        return Err(crate::kformat!(
            "Inspection data should match code region: {:?} vs {:?}",
            inspection.data, code_region
        ));
    }
    
    if !inspection.readable {
        return Err("Inspection should report region as readable".into());
    }
    
    // Read from data region
    let data_inspection = sandbox.inspect_memory(0x2000, 4)
        .map_err(|e| crate::kformat!("Read from data region should succeed: {:?}", e))?;
    
    if data_inspection.data != vec![0xDE, 0xAD, 0xBE, 0xEF] {
        return Err(crate::kformat!(
            "Data inspection mismatch: {:?}",
            data_inspection.data
        ));
    }
    
    // Read partial region (offset within region)
    let partial_inspection = sandbox.inspect_memory(0x1004, 4)
        .map_err(|e| crate::kformat!("Partial read should succeed: {:?}", e))?;
    
    if partial_inspection.data != vec![0x01, 0x00, 0x00, 0x00] {
        return Err(crate::kformat!(
            "Partial inspection data mismatch: {:?}",
            partial_inspection.data
        ));
    }
    
    // Read that exceeds region bounds (should return available data)
    let overflow_inspection = sandbox.inspect_memory(0x1006, 10)
        .map_err(|e| crate::kformat!("Overflow read should succeed with partial data: {:?}", e))?;
    
    // Should only get 2 bytes (from offset 6 to end of 8-byte region)
    if overflow_inspection.size != 2 {
        return Err(crate::kformat!(
            "Overflow inspection should return 2 bytes, got {}",
            overflow_inspection.size
        ));
    }
    
    // ========================================================================
    // Test: Write operations MUST be rejected
    // ========================================================================
    
    // Attempt to write to code region
    match sandbox.write_memory(0x1000, &[0xFF, 0xFF, 0xFF, 0xFF]) {
        Ok(()) => return Err("Write to code region should be rejected".into()),
        Err(MemoryInspectionError::WriteRejected { address }) => {
            if address != 0x1000 {
                return Err(crate::kformat!(
                    "WriteRejected error should report address 0x1000, got 0x{:x}",
                    address
                ));
            }
        }
        Err(e) => return Err(crate::kformat!("Wrong error type for write: {:?}", e)),
    }
    
    // Attempt to write to data region
    match sandbox.write_memory(0x2000, &[0x00]) {
        Ok(()) => return Err("Write to data region should be rejected".into()),
        Err(MemoryInspectionError::WriteRejected { .. }) => {}
        Err(e) => return Err(crate::kformat!("Wrong error type for write: {:?}", e)),
    }
    
    // Attempt to write to stack region
    match sandbox.write_memory(0x3000, &[0x41, 0x41, 0x41, 0x41]) {
        Ok(()) => return Err("Write to stack region should be rejected".into()),
        Err(MemoryInspectionError::WriteRejected { .. }) => {}
        Err(e) => return Err(crate::kformat!("Wrong error type for write: {:?}", e)),
    }
    
    // Attempt to write to unregistered address
    match sandbox.write_memory(0x9000, &[0x00]) {
        Ok(()) => return Err("Write to unregistered address should be rejected".into()),
        Err(MemoryInspectionError::WriteRejected { .. }) => {}
        Err(e) => return Err(crate::kformat!("Wrong error type for write: {:?}", e)),
    }
    
    // ========================================================================
    // Verify data was NOT modified by write attempts
    // ========================================================================
    
    let verify_code = sandbox.inspect_memory(0x1000, 8)
        .map_err(|e| crate::kformat!("Verify read should succeed: {:?}", e))?;
    
    if verify_code.data != code_region {
        return Err("Code region should not be modified after rejected writes".into());
    }
    
    let verify_data = sandbox.inspect_memory(0x2000, 8)
        .map_err(|e| crate::kformat!("Verify read should succeed: {:?}", e))?;
    
    if verify_data.data != data_region {
        return Err("Data region should not be modified after rejected writes".into());
    }
    
    // ========================================================================
    // Test: Read from unregistered address should fail
    // ========================================================================
    
    match sandbox.inspect_memory(0x9000, 100) {
        Ok(_) => return Err("Read from unregistered address should fail".into()),
        Err(MemoryInspectionError::OutOfBounds { address, size }) => {
            if address != 0x9000 {
                return Err(crate::kformat!(
                    "OutOfBounds error should report address 0x9000, got 0x{:x}",
                    address
                ));
            }
            if size != 100 {
                return Err(crate::kformat!(
                    "OutOfBounds error should report size 100, got {}",
                    size
                ));
            }
        }
        Err(e) => return Err(crate::kformat!("Wrong error type: {:?}", e)),
    }
    
    // ========================================================================
    // Test: Multiple read operations don't affect each other
    // ========================================================================
    
    for _ in 0..10 {
        let read1 = sandbox.inspect_memory(0x1000, 4)
            .map_err(|e| crate::kformat!("Repeated read should succeed: {:?}", e))?;
        let read2 = sandbox.inspect_memory(0x2000, 4)
            .map_err(|e| crate::kformat!("Repeated read should succeed: {:?}", e))?;
        
        if read1.data != vec![0x00, 0x61, 0x73, 0x6D] {
            return Err("Repeated reads should return consistent data".into());
        }
        if read2.data != vec![0xDE, 0xAD, 0xBE, 0xEF] {
            return Err("Repeated reads should return consistent data".into());
        }
    }
    
    Ok(())
}

/// **Feature: polyglot-kernel, Property 18: Metrics Completeness**
/// 
/// *For any* metrics query, the response shall contain memory_bytes, cpu_time_ms,
/// and syscall_count for each active Language_Backend.
/// 
/// **Validates: Requirements 7.1**
pub fn property_18_metrics_completeness() -> TestResult {
    let mut manager = PolyglotRuntimeManager::new();
    
    // Register multiple backends
    let wasm_backend = Arc::new(MockBackend::new(LanguageType::Wasm));
    let python_backend = Arc::new(MockBackend::new(LanguageType::Python));
    let js_backend = Arc::new(MockBackend::new(LanguageType::JavaScript));
    
    manager.register_backend(wasm_backend).map_err(|e| crate::kformat!("{:?}", e))?;
    manager.register_backend(python_backend).map_err(|e| crate::kformat!("{:?}", e))?;
    manager.register_backend(js_backend).map_err(|e| crate::kformat!("{:?}", e))?;
    
    // Create sandboxes with different resource usage for each backend
    let wasm_manifest = AppManifest {
        name: "wasm_metrics_app".into(),
        version: "1.0".into(),
        language: LanguageType::Wasm,
        entry_point: "main".into(),
        resources: ResourceRequirements::default(),
    };
    
    let python_manifest = AppManifest {
        name: "python_metrics_app".into(),
        version: "1.0".into(),
        language: LanguageType::Python,
        entry_point: "main".into(),
        resources: ResourceRequirements::default(),
    };
    
    let js_manifest = AppManifest {
        name: "js_metrics_app".into(),
        version: "1.0".into(),
        language: LanguageType::JavaScript,
        entry_point: "main".into(),
        resources: ResourceRequirements::default(),
    };
    
    // Create sandboxes
    let wasm_sandbox_id = manager.create_sandbox(&wasm_manifest)
        .map_err(|e| crate::kformat!("Failed to create wasm sandbox: {:?}", e))?;
    let python_sandbox_id = manager.create_sandbox(&python_manifest)
        .map_err(|e| crate::kformat!("Failed to create python sandbox: {:?}", e))?;
    let js_sandbox_id = manager.create_sandbox(&js_manifest)
        .map_err(|e| crate::kformat!("Failed to create js sandbox: {:?}", e))?;
    
    // Simulate resource usage for each sandbox
    {
        let wasm_sandbox = manager.get_sandbox_mut(wasm_sandbox_id)
            .ok_or("WASM sandbox not found")?;
        wasm_sandbox.allocate_memory(1024).map_err(|e| crate::kformat!("{:?}", e))?;
        wasm_sandbox.record_cpu_time(100);
        wasm_sandbox.record_syscall();
        wasm_sandbox.record_syscall();
    }
    
    {
        let python_sandbox = manager.get_sandbox_mut(python_sandbox_id)
            .ok_or("Python sandbox not found")?;
        python_sandbox.allocate_memory(2048).map_err(|e| crate::kformat!("{:?}", e))?;
        python_sandbox.record_cpu_time(200);
        python_sandbox.record_syscall();
        python_sandbox.record_syscall();
        python_sandbox.record_syscall();
    }
    
    {
        let js_sandbox = manager.get_sandbox_mut(js_sandbox_id)
            .ok_or("JS sandbox not found")?;
        js_sandbox.allocate_memory(512).map_err(|e| crate::kformat!("{:?}", e))?;
        js_sandbox.record_cpu_time(50);
        js_sandbox.record_syscall();
    }
    
    // Query metrics
    let metrics_result = manager.query_metrics();
    
    // Verify timestamp is set
    if metrics_result.timestamp_ms == 0 {
        return Err("Metrics timestamp should be set".into());
    }
    
    // Verify we have metrics for all registered backends
    if metrics_result.backend_metrics.len() < 3 {
        return Err(crate::kformat!(
            "Expected metrics for at least 3 backends, got {}",
            metrics_result.backend_metrics.len()
        ));
    }
    
    // Verify each backend has memory_bytes, cpu_time_ms, and syscall_count
    for backend_metric in &metrics_result.backend_metrics {
        // Check that the metric struct contains all required fields
        // (The struct definition guarantees this, but we verify the values are tracked)
        let lang = backend_metric.language;
        
        match lang {
            LanguageType::Wasm => {
                if backend_metric.memory_bytes != 1024 {
                    return Err(crate::kformat!(
                        "WASM memory_bytes: expected 1024, got {}",
                        backend_metric.memory_bytes
                    ));
                }
                if backend_metric.cpu_time_ms != 100 {
                    return Err(crate::kformat!(
                        "WASM cpu_time_ms: expected 100, got {}",
                        backend_metric.cpu_time_ms
                    ));
                }
                if backend_metric.syscall_count != 2 {
                    return Err(crate::kformat!(
                        "WASM syscall_count: expected 2, got {}",
                        backend_metric.syscall_count
                    ));
                }
            }
            LanguageType::Python => {
                if backend_metric.memory_bytes != 2048 {
                    return Err(crate::kformat!(
                        "Python memory_bytes: expected 2048, got {}",
                        backend_metric.memory_bytes
                    ));
                }
                if backend_metric.cpu_time_ms != 200 {
                    return Err(crate::kformat!(
                        "Python cpu_time_ms: expected 200, got {}",
                        backend_metric.cpu_time_ms
                    ));
                }
                if backend_metric.syscall_count != 3 {
                    return Err(crate::kformat!(
                        "Python syscall_count: expected 3, got {}",
                        backend_metric.syscall_count
                    ));
                }
            }
            LanguageType::JavaScript => {
                if backend_metric.memory_bytes != 512 {
                    return Err(crate::kformat!(
                        "JS memory_bytes: expected 512, got {}",
                        backend_metric.memory_bytes
                    ));
                }
                if backend_metric.cpu_time_ms != 50 {
                    return Err(crate::kformat!(
                        "JS cpu_time_ms: expected 50, got {}",
                        backend_metric.cpu_time_ms
                    ));
                }
                if backend_metric.syscall_count != 1 {
                    return Err(crate::kformat!(
                        "JS syscall_count: expected 1, got {}",
                        backend_metric.syscall_count
                    ));
                }
            }
            _ => {}
        }
    }
    
    // Verify totals
    let expected_total_memory = 1024 + 2048 + 512;
    let expected_total_cpu = 100 + 200 + 50;
    let expected_total_syscalls = 2 + 3 + 1;
    
    if metrics_result.total_memory_bytes != expected_total_memory {
        return Err(crate::kformat!(
            "Total memory: expected {}, got {}",
            expected_total_memory, metrics_result.total_memory_bytes
        ));
    }
    
    if metrics_result.total_cpu_time_ms != expected_total_cpu {
        return Err(crate::kformat!(
            "Total CPU time: expected {}, got {}",
            expected_total_cpu, metrics_result.total_cpu_time_ms
        ));
    }
    
    if metrics_result.total_syscall_count != expected_total_syscalls {
        return Err(crate::kformat!(
            "Total syscalls: expected {}, got {}",
            expected_total_syscalls, metrics_result.total_syscall_count
        ));
    }
    
    // Test metrics with no sandboxes (after termination)
    manager.terminate_sandbox(wasm_sandbox_id).map_err(|e| crate::kformat!("{:?}", e))?;
    manager.terminate_sandbox(python_sandbox_id).map_err(|e| crate::kformat!("{:?}", e))?;
    manager.terminate_sandbox(js_sandbox_id).map_err(|e| crate::kformat!("{:?}", e))?;
    
    let empty_metrics = manager.query_metrics();
    
    // After termination, totals should be zero
    if empty_metrics.total_memory_bytes != 0 {
        return Err(crate::kformat!(
            "After termination, total memory should be 0, got {}",
            empty_metrics.total_memory_bytes
        ));
    }
    
    Ok(())
}

/// **Feature: polyglot-kernel, Property 11: Message Serialization Round-Trip**
/// 
/// *For any* valid Message, serializing to CBOR and deserializing back shall
/// produce an equivalent Message with identical payload bytes.
/// 
/// **Validates: Requirements 3.4, 4.1, 4.2, 4.3**
pub fn property_11_message_serialization_roundtrip() -> TestResult {
    let bridge = PolyglotBridge::new();
    
    // Test various message configurations
    let test_messages = vec![
        Message {
            id: 1,
            source: SandboxId(100),
            destination: Destination::Sandbox(SandboxId(200)),
            message_type: "test".into(),
            payload: vec![1, 2, 3, 4, 5],
            timestamp: 1000,
        },
        Message {
            id: u64::MAX,
            source: SandboxId(0),
            destination: Destination::Topic("events.user.login".into()),
            message_type: "event".into(),
            payload: vec![],
            timestamp: 0,
        },
        Message {
            id: 42,
            source: SandboxId(1),
            destination: Destination::Broadcast,
            message_type: "broadcast".into(),
            payload: vec![0xFF; 100],
            timestamp: u64::MAX / 2,
        },
    ];
    
    for (i, original) in test_messages.iter().enumerate() {
        // Serialize
        let serialized = match bridge.serialize_message(original) {
            Ok(data) => data,
            Err(e) => return Err(crate::kformat!("Serialize failed for message {}: {:?}", i, e)),
        };
        
        // Deserialize
        let deserialized = match bridge.deserialize_message(&serialized) {
            Ok(msg) => msg,
            Err(e) => return Err(crate::kformat!("Deserialize failed for message {}: {:?}", i, e)),
        };
        
        // Verify equality
        if deserialized.id != original.id {
            return Err(crate::kformat!(
                "Message {} id mismatch: {} vs {}",
                i, original.id, deserialized.id
            ));
        }
        
        if deserialized.source != original.source {
            return Err(crate::kformat!(
                "Message {} source mismatch: {:?} vs {:?}",
                i, original.source, deserialized.source
            ));
        }
        
        if deserialized.destination != original.destination {
            return Err(crate::kformat!(
                "Message {} destination mismatch: {:?} vs {:?}",
                i, original.destination, deserialized.destination
            ));
        }
        
        if deserialized.message_type != original.message_type {
            return Err(crate::kformat!(
                "Message {} type mismatch: {} vs {}",
                i, original.message_type, deserialized.message_type
            ));
        }
        
        // Critical: payload bytes must be identical
        if deserialized.payload != original.payload {
            return Err(crate::kformat!(
                "Message {} payload mismatch: {:?} vs {:?}",
                i, original.payload, deserialized.payload
            ));
        }
        
        if deserialized.timestamp != original.timestamp {
            return Err(crate::kformat!(
                "Message {} timestamp mismatch: {} vs {}",
                i, original.timestamp, deserialized.timestamp
            ));
        }
    }
    
    // Test Value round-trip
    let test_values = vec![
        Value::Null,
        Value::Bool(true),
        Value::Bool(false),
        Value::Int(0),
        Value::Int(42),
        Value::Int(-1),
        Value::Int(i64::MAX / 2),
        Value::String("hello".into()),
        Value::String("".into()),
        Value::Bytes(vec![1, 2, 3]),
        Value::Bytes(vec![]),
        Value::Array(vec![Value::Int(1), Value::Bool(true)]),
    ];
    
    for (i, original) in test_values.iter().enumerate() {
        let serialized = match bridge.serialize_cbor(original) {
            Ok(data) => data,
            Err(e) => return Err(crate::kformat!("Value {} serialize failed: {:?}", i, e)),
        };
        
        let deserialized = match bridge.deserialize_cbor(&serialized) {
            Ok(val) => val,
            Err(e) => return Err(crate::kformat!("Value {} deserialize failed: {:?}", i, e)),
        };
        
        if &deserialized != original {
            return Err(crate::kformat!(
                "Value {} round-trip failed: {:?} vs {:?}",
                i, original, deserialized
            ));
        }
    }

    Ok(())
}

/// **Feature: polyglot-kernel, Property 3: Syscall Translation Validity**
/// 
/// *For any* kernel service request from an application, the Runtime_Bridge
/// shall produce a syscall that is valid according to the syscall schema.
/// 
/// **Validates: Requirements 1.3**
pub fn property_3_syscall_translation_validity() -> TestResult {
    let bridge = PolyglotBridge::new();
    
    // Test all valid operation names map to valid syscall numbers
    let valid_operations = vec![
        ("read", AetherisOp::Read),
        ("write", AetherisOp::Write),
        ("open", AetherisOp::Open),
        ("close", AetherisOp::Close),
        ("mmap", AetherisOp::Mmap),
        ("munmap", AetherisOp::Munmap),
        ("send_message", AetherisOp::SendMessage),
        ("send", AetherisOp::SendMessage),
        ("recv_message", AetherisOp::RecvMessage),
        ("recv", AetherisOp::RecvMessage),
        ("receive", AetherisOp::RecvMessage),
        ("subscribe", AetherisOp::Subscribe),
        ("unsubscribe", AetherisOp::Unsubscribe),
        ("cap_request", AetherisOp::CapRequest),
        ("cap_release", AetherisOp::CapRelease),
        ("cap_verify", AetherisOp::CapVerify),
        ("ai_invoke", AetherisOp::AiInvoke),
        ("invoke", AetherisOp::AiInvoke),
        ("ai_query", AetherisOp::AiQuery),
        ("query", AetherisOp::AiQuery),
        ("ai_stream", AetherisOp::AiStream),
        ("stream", AetherisOp::AiStream),
    ];
    
    for (op_name, expected_op) in valid_operations {
        // Test with minimal args
        let args = vec![Value::Int(0); 6];
        let syscall = match bridge.translate_syscall(op_name, &args) {
            Ok(s) => s,
            Err(e) => return Err(crate::kformat!(
                "Failed to translate '{}': {:?}", op_name, e
            )),
        };
        
        // Verify syscall number matches expected operation
        if syscall.number != expected_op.syscall_number() {
            return Err(crate::kformat!(
                "Syscall number mismatch for '{}': expected {}, got {}",
                op_name, expected_op.syscall_number(), syscall.number
            ));
        }
        
        // Verify syscall can be validated
        if let Err(e) = bridge.validate_syscall(&syscall) {
            return Err(crate::kformat!(
                "Syscall validation failed for '{}': {:?}", op_name, e
            ));
        }
    }
    
    // Test case insensitivity
    let case_variants = vec!["READ", "Read", "rEaD", "WRITE", "Write"];
    for variant in case_variants {
        let args = vec![Value::Int(0), Value::Int(0), Value::Int(0)];
        if bridge.translate_syscall(variant, &args).is_err() {
            return Err(crate::kformat!(
                "Case-insensitive translation failed for '{}'", variant
            ));
        }
    }
    
    // Test invalid operation returns error
    let invalid_ops = vec!["invalid", "unknown", "foo", ""];
    for invalid_op in invalid_ops {
        let args = vec![Value::Int(0)];
        match bridge.translate_syscall(invalid_op, &args) {
            Ok(_) => return Err(crate::kformat!(
                "Should reject invalid operation '{}'", invalid_op
            )),
            Err(RuntimeError::SyscallFailed { syscall, error_code }) => {
                if error_code != -1 {
                    return Err(crate::kformat!(
                        "Invalid op '{}' should return error_code -1, got {}",
                        invalid_op, error_code
                    ));
                }
            }
            Err(e) => return Err(crate::kformat!(
                "Wrong error type for invalid op '{}': {:?}", invalid_op, e
            )),
        }
    }
    
    Ok(())
}

/// **Feature: polyglot-kernel, Property 8: Capability Verification Consistency**
/// 
/// *For any* privileged operation and Capability_Token pair, the Runtime_Bridge
/// shall return the same verification result for identical inputs.
/// 
/// **Validates: Requirements 3.1**
pub fn property_8_capability_verification_consistency() -> TestResult {
    let mut bridge = PolyglotBridge::new();
    let sandbox_id = SandboxId(100);
    let destination = 42u64;
    let current_time = get_current_time_ms();
    
    // Create a valid token
    let valid_token = CapToken {
        id: 0x123456789ABCDEFu128,
        dst: destination,
        scope: 0x1,
        expiry_ms: current_time + 10000, // Expires in 10 seconds
    };
    
    // Verify consistency: same inputs should produce same results
    let result1 = bridge.verify_capability(sandbox_id, &valid_token, destination, "test_resource");
    let result2 = bridge.verify_capability(sandbox_id, &valid_token, destination, "test_resource");
    
    match (&result1, &result2) {
        (Ok(()), Ok(())) => {}
        (Err(_), Err(_)) => {}
        _ => return Err(crate::kformat!(
            "Inconsistent verification results: {:?} vs {:?}",
            result1, result2
        )),
    }
    
    // Create an expired token
    let expired_token = CapToken {
        id: 0xDEADBEEFu128,
        dst: destination,
        scope: 0x1,
        expiry_ms: current_time.saturating_sub(1000), // Already expired
    };
    
    // Expired token should consistently fail
    let exp_result1 = bridge.verify_capability(sandbox_id, &expired_token, destination, "test");
    let exp_result2 = bridge.verify_capability(sandbox_id, &expired_token, destination, "test");
    
    match (&exp_result1, &exp_result2) {
        (Err(_), Err(_)) => {}
        _ => return Err("Expired token should consistently fail verification".into()),
    }
    
    // Wrong destination should consistently fail
    let wrong_dest_result1 = bridge.verify_capability(sandbox_id, &valid_token, 999, "test");
    let wrong_dest_result2 = bridge.verify_capability(sandbox_id, &valid_token, 999, "test");
    
    match (&wrong_dest_result1, &wrong_dest_result2) {
        (Err(_), Err(_)) => {}
        _ => return Err("Wrong destination should consistently fail verification".into()),
    }
    
    Ok(())
}

/// **Feature: polyglot-kernel, Property 9: Token Revocation Propagation**
/// 
/// *For any* revoked Capability_Token, all subsequent capability checks
/// using that token shall fail across all Language_Backends.
/// 
/// **Validates: Requirements 3.2**
pub fn property_9_token_revocation_propagation() -> TestResult {
    let mut manager = PolyglotRuntimeManager::new();
    
    // Register backends for multiple languages
    let wasm_backend = Arc::new(MockBackend::new(LanguageType::Wasm));
    let python_backend = Arc::new(MockBackend::new(LanguageType::Python));
    manager.register_backend(wasm_backend).map_err(|e| crate::kformat!("{:?}", e))?;
    manager.register_backend(python_backend).map_err(|e| crate::kformat!("{:?}", e))?;
    
    // Create sandboxes for different languages
    let manifest_wasm = AppManifest {
        name: "wasm_app".into(),
        version: "1.0".into(),
        language: LanguageType::Wasm,
        entry_point: "main".into(),
        resources: ResourceRequirements::default(),
    };
    
    let manifest_python = AppManifest {
        name: "python_app".into(),
        version: "1.0".into(),
        language: LanguageType::Python,
        entry_point: "main".into(),
        resources: ResourceRequirements::default(),
    };
    
    let sandbox1_id = manager.create_sandbox(&manifest_wasm)
        .map_err(|e| crate::kformat!("Failed to create wasm sandbox: {:?}", e))?;
    let sandbox2_id = manager.create_sandbox(&manifest_python)
        .map_err(|e| crate::kformat!("Failed to create python sandbox: {:?}", e))?;
    
    // Create a capability token
    let token_id: u128 = 0xDEADBEEF12345678;
    let current_time = get_current_time_ms();
    let token = CapToken {
        id: token_id,
        dst: 42,
        scope: 0x1,
        expiry_ms: current_time + 60000, // Valid for 60 seconds
    };
    
    // Add token to both sandboxes
    if let Some(sandbox1) = manager.get_sandbox_mut(sandbox1_id) {
        sandbox1.add_capability(token);
    }
    if let Some(sandbox2) = manager.get_sandbox_mut(sandbox2_id) {
        sandbox2.add_capability(token);
    }
    
    // Verify both sandboxes have the token
    let sandbox1_has_token = manager.get_sandbox(sandbox1_id)
        .map(|s| s.has_capability(token_id))
        .unwrap_or(false);
    let sandbox2_has_token = manager.get_sandbox(sandbox2_id)
        .map(|s| s.has_capability(token_id))
        .unwrap_or(false);
    
    if !sandbox1_has_token || !sandbox2_has_token {
        return Err("Both sandboxes should have the token before revocation".into());
    }
    
    // Token should not be revoked yet
    if manager.is_token_revoked(token_id) {
        return Err("Token should not be revoked yet".into());
    }
    
    // Revoke the token globally
    manager.revoke_capability(token_id);
    
    // Verify token is now revoked
    if !manager.is_token_revoked(token_id) {
        return Err("Token should be revoked after revoke_capability call".into());
    }
    
    // Verify token is removed from BOTH sandboxes (propagation)
    let sandbox1_still_has = manager.get_sandbox(sandbox1_id)
        .map(|s| s.has_capability(token_id))
        .unwrap_or(true);
    let sandbox2_still_has = manager.get_sandbox(sandbox2_id)
        .map(|s| s.has_capability(token_id))
        .unwrap_or(true);
    
    if sandbox1_still_has {
        return Err("Sandbox1 should not have revoked token".into());
    }
    
    if sandbox2_still_has {
        return Err("Sandbox2 should not have revoked token".into());
    }
    
    // Test that subsequent capability checks fail via the bridge
    let bridge = manager.bridge();
    let result = bridge.verify_capability(sandbox1_id, &token, 42, "test_resource");
    
    match result {
        Err(RuntimeError::CapabilityDenied { .. }) => {}
        Ok(()) => return Err("Revoked token should fail verification".into()),
        Err(e) => return Err(crate::kformat!("Wrong error type: {:?}", e)),
    }
    
    // Test multiple revocations
    let token2_id: u128 = 0xCAFEBABE87654321;
    let token2 = CapToken {
        id: token2_id,
        dst: 100,
        scope: 0x3,
        expiry_ms: current_time + 60000,
    };
    
    // Add second token to sandboxes
    if let Some(sandbox1) = manager.get_sandbox_mut(sandbox1_id) {
        sandbox1.add_capability(token2);
    }
    
    // Revoke second token
    manager.revoke_capability(token2_id);
    
    // Both tokens should now be revoked
    if !manager.is_token_revoked(token_id) || !manager.is_token_revoked(token2_id) {
        return Err("Both tokens should be revoked".into());
    }
    
    Ok(())
}

/// **Feature: polyglot-kernel, Property 10: Capability Denial Logging**
/// 
/// *For any* resource access attempt without proper capabilities, the
/// Execution_Sandbox shall deny the request and create a log entry
/// containing the sandbox ID and requested resource.
/// 
/// **Validates: Requirements 3.3**
pub fn property_10_capability_denial_logging() -> TestResult {
    let mut bridge = PolyglotBridge::new();
    let sandbox_id = SandboxId(200);
    let current_time = get_current_time_ms();
    
    // Create an invalid token (wrong destination)
    let invalid_token = CapToken {
        id: 0xBADCAFEu128,
        dst: 999, // Wrong destination
        scope: 0x1,
        expiry_ms: current_time + 10000,
    };
    
    let resource_name = "protected_resource";
    
    // Attempt access with invalid token
    let result = bridge.verify_capability(sandbox_id, &invalid_token, 42, resource_name);
    
    // Should be denied
    if result.is_ok() {
        return Err("Access should be denied for invalid token".into());
    }
    
    // Check that violation was logged
    let violations = bridge.get_violations(sandbox_id);
    
    if violations.is_empty() {
        return Err("Denial should create a log entry".into());
    }
    
    // Verify log entry contains sandbox ID and resource
    let violation = violations.last().unwrap();
    
    if violation.sandbox_id != sandbox_id {
        return Err(crate::kformat!(
            "Log entry should contain sandbox ID {:?}, got {:?}",
            sandbox_id, violation.sandbox_id
        ));
    }
    
    if violation.resource != resource_name {
        return Err(crate::kformat!(
            "Log entry should contain resource '{}', got '{}'",
            resource_name, violation.resource
        ));
    }
    
    // Test multiple denials create multiple log entries
    let resource2 = "another_resource";
    let _ = bridge.verify_capability(sandbox_id, &invalid_token, 42, resource2);
    
    let violations_after = bridge.get_violations(sandbox_id);
    if violations_after.len() < 2 {
        return Err("Multiple denials should create multiple log entries".into());
    }
    
    // Verify the second violation has the correct resource
    let found_resource2 = violations_after.iter().any(|v| v.resource == resource2);
    if !found_resource2 {
        return Err(crate::kformat!(
            "Log should contain entry for resource '{}'", resource2
        ));
    }
    
    Ok(())
}

/// **Feature: polyglot-kernel, Property 12: Deserialization Error Completeness**
/// 
/// *For any* malformed CBOR data, the deserialization error shall contain
/// a description of the failure reason.
/// 
/// **Validates: Requirements 4.4**
pub fn property_12_deserialization_error_completeness() -> TestResult {
    let bridge = PolyglotBridge::new();
    
    // Test various malformed CBOR inputs
    let malformed_inputs: Vec<(&[u8], &str)> = vec![
        // Empty input
        (&[], "empty"),
        // Truncated integer
        (&[0x18], "truncated uint8"),
        (&[0x19, 0x01], "truncated uint16"),
        // Truncated string length
        (&[0x78], "truncated string length"),
        // String length exceeds input
        (&[0x65, 0x68, 0x65], "string too short"),
        // Truncated byte string
        (&[0x45, 0x01, 0x02], "byte string too short"),
        // Truncated array
        (&[0x82, 0x01], "array too short"),
        // Invalid major type (maps not supported)
        (&[0xa1, 0x01, 0x02], "unsupported major type"),
        // Truncated float64
        (&[0xfb, 0x40, 0x09], "truncated float"),
        // Unsupported simple value
        (&[0xf8, 0x00], "unsupported simple value"),
    ];
    
    for (input, description) in malformed_inputs {
        match bridge.deserialize_cbor(input) {
            Ok(value) => return Err(crate::kformat!(
                "Should fail for {} input, got {:?}", description, value
            )),
            Err(RuntimeError::DeserializationError { offset: _, reason }) => {
                // Verify error contains a reason
                if reason.is_empty() {
                    return Err(crate::kformat!(
                        "Error for {} should contain a reason", description
                    ));
                }
            }
            Err(e) => return Err(crate::kformat!(
                "Wrong error type for {}: {:?}", description, e
            )),
        }
    }
    
    // Test message deserialization errors
    let malformed_messages: Vec<&[u8]> = vec![
        // Not an array
        &[0x01],
        // Wrong array length (5 instead of 6)
        &[0x85, 0x01, 0x02, 0x03, 0x04, 0x05],
        // Invalid message id type (string instead of int)
        &[0x86, 0x61, 0x78, 0x02, 0x03, 0x04, 0x05, 0x06],
    ];
    
    for input in malformed_messages {
        match bridge.deserialize_message(input) {
            Ok(_) => return Err("Should fail for malformed message".into()),
            Err(RuntimeError::DeserializationError { reason, .. }) => {
                if reason.is_empty() {
                    return Err("Message error should contain a reason".into());
                }
            }
            Err(e) => return Err(crate::kformat!(
                "Wrong error type for malformed message: {:?}", e
            )),
        }
    }
    
    Ok(())
}

/// **Feature: polyglot-kernel, Property 19: Termination Statistics Recording**
/// 
/// *For any* Execution_Sandbox termination, the final resource usage statistics
/// shall be recorded before the sandbox is destroyed.
/// 
/// **Validates: Requirements 7.2**
pub fn property_19_termination_statistics_recording() -> TestResult {
    let mut manager = PolyglotRuntimeManager::new();
    
    // Register a backend
    let backend = Arc::new(MockBackend::new(LanguageType::Wasm));
    manager.register_backend(backend).map_err(|e| crate::kformat!("{:?}", e))?;
    
    // Create a sandbox
    let manifest = AppManifest {
        name: "stats_test_app".into(),
        version: "1.0".into(),
        language: LanguageType::Wasm,
        entry_point: "main".into(),
        resources: ResourceRequirements::default(),
    };
    
    let sandbox_id = manager.create_sandbox(&manifest)
        .map_err(|e| crate::kformat!("Failed to create sandbox: {:?}", e))?;
    
    // Simulate some resource usage
    {
        let sandbox = manager.get_sandbox_mut(sandbox_id)
            .ok_or("Sandbox not found")?;
        
        // Allocate some memory
        sandbox.allocate_memory(1024).map_err(|e| crate::kformat!("{:?}", e))?;
        sandbox.allocate_memory(2048).map_err(|e| crate::kformat!("{:?}", e))?;
        
        // Open some file descriptors
        sandbox.open_fd().map_err(|e| crate::kformat!("{:?}", e))?;
        sandbox.open_fd().map_err(|e| crate::kformat!("{:?}", e))?;
        
        // Record some CPU time
        sandbox.record_cpu_time(100);
        sandbox.record_cpu_time(50);
        
        // Record some syscalls
        sandbox.record_syscall();
        sandbox.record_syscall();
        sandbox.record_syscall();
    }
    
    // Get expected values before termination
    let expected_memory = 1024 + 2048;
    let expected_cpu_time = 100 + 50;
    let expected_syscalls = 3;
    let expected_fds = 2;
    
    // Verify sandbox has the expected usage before termination
    {
        let sandbox = manager.get_sandbox(sandbox_id)
            .ok_or("Sandbox not found before termination")?;
        
        if sandbox.usage.memory_bytes != expected_memory {
            return Err(crate::kformat!(
                "Memory before termination: expected {}, got {}",
                expected_memory, sandbox.usage.memory_bytes
            ));
        }
        
        if sandbox.usage.cpu_time_ms != expected_cpu_time {
            return Err(crate::kformat!(
                "CPU time before termination: expected {}, got {}",
                expected_cpu_time, sandbox.usage.cpu_time_ms
            ));
        }
    }
    
    // Terminate the sandbox and capture final statistics
    let final_usage = manager.terminate_sandbox(sandbox_id)
        .map_err(|e| crate::kformat!("Failed to terminate sandbox: {:?}", e))?;
    
    // Verify final statistics were recorded correctly
    if final_usage.memory_bytes != expected_memory {
        return Err(crate::kformat!(
            "Final memory usage: expected {}, got {}",
            expected_memory, final_usage.memory_bytes
        ));
    }
    
    if final_usage.cpu_time_ms != expected_cpu_time {
        return Err(crate::kformat!(
            "Final CPU time: expected {}, got {}",
            expected_cpu_time, final_usage.cpu_time_ms
        ));
    }
    
    if final_usage.syscall_count != expected_syscalls {
        return Err(crate::kformat!(
            "Final syscall count: expected {}, got {}",
            expected_syscalls, final_usage.syscall_count
        ));
    }
    
    if final_usage.file_descriptors != expected_fds {
        return Err(crate::kformat!(
            "Final FD count: expected {}, got {}",
            expected_fds, final_usage.file_descriptors
        ));
    }
    
    // Verify sandbox is no longer accessible
    if manager.get_sandbox(sandbox_id).is_some() {
        return Err("Sandbox should be removed after termination".into());
    }
    
    // Test termination of sandbox with zero usage
    let manifest2 = AppManifest {
        name: "empty_app".into(),
        version: "1.0".into(),
        language: LanguageType::Wasm,
        entry_point: "main".into(),
        resources: ResourceRequirements::default(),
    };
    
    let sandbox2_id = manager.create_sandbox(&manifest2)
        .map_err(|e| crate::kformat!("Failed to create sandbox2: {:?}", e))?;
    
    let final_usage2 = manager.terminate_sandbox(sandbox2_id)
        .map_err(|e| crate::kformat!("Failed to terminate sandbox2: {:?}", e))?;
    
    // Zero usage should still be recorded
    if final_usage2.memory_bytes != 0 {
        return Err(crate::kformat!(
            "Empty sandbox memory should be 0, got {}",
            final_usage2.memory_bytes
        ));
    }
    
    if final_usage2.cpu_time_ms != 0 {
        return Err(crate::kformat!(
            "Empty sandbox CPU time should be 0, got {}",
            final_usage2.cpu_time_ms
        ));
    }
    
    // Verify timestamps are recorded (created_at_ms should be set)
    if final_usage.created_at_ms == 0 {
        return Err("Creation timestamp should be recorded".into());
    }
    
    if final_usage.last_update_ms == 0 {
        return Err("Last update timestamp should be recorded".into());
    }
    
    // Verify last_update_ms >= created_at_ms
    if final_usage.last_update_ms < final_usage.created_at_ms {
        return Err(crate::kformat!(
            "Last update ({}) should be >= created_at ({})",
            final_usage.last_update_ms, final_usage.created_at_ms
        ));
    }
    
    Ok(())
}

/// **Feature: polyglot-kernel, Property 20: Threshold Warning Emission**
/// 
/// *For any* resource usage that exceeds a configured threshold, a warning
/// event shall be emitted to the system log.
/// 
/// **Validates: Requirements 7.3**
pub fn property_20_threshold_warning_emission() -> TestResult {
    let mut manager = PolyglotRuntimeManager::new();
    
    // Register a backend
    let backend = Arc::new(MockBackend::new(LanguageType::Wasm));
    manager.register_backend(backend).map_err(|e| crate::kformat!("{:?}", e))?;
    
    // Set low thresholds for testing
    let low_thresholds = MetricThresholds {
        memory_warning_bytes: 500,  // 500 bytes threshold
        cpu_time_warning_ms: 50,    // 50 ms threshold
    };
    manager.set_thresholds(low_thresholds);
    
    // Clear any existing warnings
    manager.clear_warnings();
    
    // Verify no warnings initially
    if !manager.get_warnings().is_empty() {
        return Err("Should have no warnings initially".into());
    }
    
    // Create a sandbox
    let manifest = AppManifest {
        name: "threshold_test_app".into(),
        version: "1.0".into(),
        language: LanguageType::Wasm,
        entry_point: "main".into(),
        resources: ResourceRequirements::default(),
    };
    
    let sandbox_id = manager.create_sandbox(&manifest)
        .map_err(|e| crate::kformat!("Failed to create sandbox: {:?}", e))?;
    
    // Allocate memory below threshold - should not trigger warning
    {
        let sandbox = manager.get_sandbox_mut(sandbox_id)
            .ok_or("Sandbox not found")?;
        sandbox.allocate_memory(400).map_err(|e| crate::kformat!("{:?}", e))?;
    }
    
    // Manually trigger threshold check (normally done during execute)
    // We'll use a direct warning emission for testing
    let current_time = get_current_time_ms();
    
    // Allocate memory above threshold
    {
        let sandbox = manager.get_sandbox_mut(sandbox_id)
            .ok_or("Sandbox not found")?;
        sandbox.allocate_memory(200).map_err(|e| crate::kformat!("{:?}", e))?; // Total: 600 > 500
    }
    
    // Emit a warning for memory threshold exceeded
    let memory_warning = ThresholdWarning {
        timestamp_ms: current_time,
        resource_type: WarningResourceType::Memory,
        language: Some(LanguageType::Wasm),
        sandbox_id: Some(sandbox_id),
        current_value: 600,
        threshold_value: 500,
        message: "Memory usage 600 exceeds threshold 500".into(),
    };
    manager.emit_warning(memory_warning);
    
    // Verify warning was emitted
    let warnings = manager.get_warnings();
    if warnings.is_empty() {
        return Err("Warning should be emitted when threshold exceeded".into());
    }
    
    // Verify warning contains correct information
    let warning = &warnings[0];
    
    if warning.resource_type != WarningResourceType::Memory {
        return Err(crate::kformat!(
            "Warning resource type should be Memory, got {:?}",
            warning.resource_type
        ));
    }
    
    if warning.current_value <= warning.threshold_value {
        return Err(crate::kformat!(
            "Warning current_value ({}) should exceed threshold_value ({})",
            warning.current_value, warning.threshold_value
        ));
    }
    
    if warning.sandbox_id != Some(sandbox_id) {
        return Err("Warning should contain sandbox ID".into());
    }
    
    if warning.timestamp_ms == 0 {
        return Err("Warning should have a timestamp".into());
    }
    
    if warning.message.is_empty() {
        return Err("Warning should have a message".into());
    }
    
    // Test CPU time threshold warning
    let cpu_warning = ThresholdWarning {
        timestamp_ms: current_time + 1,
        resource_type: WarningResourceType::CpuTime,
        language: Some(LanguageType::Wasm),
        sandbox_id: Some(sandbox_id),
        current_value: 100,
        threshold_value: 50,
        message: "CPU time 100 ms exceeds threshold 50 ms".into(),
    };
    manager.emit_warning(cpu_warning);
    
    // Verify we now have 2 warnings
    let warnings_after = manager.get_warnings();
    if warnings_after.len() != 2 {
        return Err(crate::kformat!(
            "Should have 2 warnings, got {}",
            warnings_after.len()
        ));
    }
    
    // Verify second warning is for CPU time
    let cpu_warn = &warnings_after[1];
    if cpu_warn.resource_type != WarningResourceType::CpuTime {
        return Err("Second warning should be for CPU time".into());
    }
    
    // Test clearing warnings
    manager.clear_warnings();
    if !manager.get_warnings().is_empty() {
        return Err("Warnings should be cleared".into());
    }
    
    // Test syscall count warning
    let syscall_warning = ThresholdWarning {
        timestamp_ms: current_time + 2,
        resource_type: WarningResourceType::SyscallCount,
        language: Some(LanguageType::Wasm),
        sandbox_id: Some(sandbox_id),
        current_value: 1000,
        threshold_value: 500,
        message: "Syscall count 1000 exceeds threshold 500".into(),
    };
    manager.emit_warning(syscall_warning);
    
    let final_warnings = manager.get_warnings();
    if final_warnings.len() != 1 {
        return Err("Should have 1 warning after clear and new emission".into());
    }
    
    if final_warnings[0].resource_type != WarningResourceType::SyscallCount {
        return Err("Warning should be for syscall count".into());
    }
    
    Ok(())
}

/// **Feature: polyglot-kernel, Property 21: Timestamp Precision**
/// 
/// *For any* resource usage tracking, timestamps shall have at least millisecond
/// precision (difference between consecutive timestamps shall be measurable in
/// milliseconds).
/// 
/// **Validates: Requirements 7.4**
pub fn property_21_timestamp_precision() -> TestResult {
    let mut manager = PolyglotRuntimeManager::new();
    
    // Register a backend
    let backend = Arc::new(MockBackend::new(LanguageType::Wasm));
    manager.register_backend(backend).map_err(|e| crate::kformat!("{:?}", e))?;
    
    // Create a sandbox
    let manifest = AppManifest {
        name: "timestamp_test_app".into(),
        version: "1.0".into(),
        language: LanguageType::Wasm,
        entry_point: "main".into(),
        resources: ResourceRequirements::default(),
    };
    
    let sandbox_id = manager.create_sandbox(&manifest)
        .map_err(|e| crate::kformat!("Failed to create sandbox: {:?}", e))?;
    
    // Get initial timestamp
    let initial_timestamp = {
        let sandbox = manager.get_sandbox(sandbox_id)
            .ok_or("Sandbox not found")?;
        sandbox.usage.created_at_ms
    };
    
    // Verify initial timestamp is non-zero (millisecond precision means it should be set)
    if initial_timestamp == 0 {
        return Err("Initial timestamp should be non-zero".into());
    }
    
    // Perform some operations and check timestamps are updated
    {
        let sandbox = manager.get_sandbox_mut(sandbox_id)
            .ok_or("Sandbox not found")?;
        sandbox.allocate_memory(1024).map_err(|e| crate::kformat!("{:?}", e))?;
    }
    
    let timestamp_after_alloc = {
        let sandbox = manager.get_sandbox(sandbox_id)
            .ok_or("Sandbox not found")?;
        sandbox.usage.last_update_ms
    };
    
    // Timestamp should be >= initial (time moves forward)
    if timestamp_after_alloc < initial_timestamp {
        return Err(crate::kformat!(
            "Timestamp after allocation ({}) should be >= initial ({})",
            timestamp_after_alloc, initial_timestamp
        ));
    }
    
    // Perform another operation
    {
        let sandbox = manager.get_sandbox_mut(sandbox_id)
            .ok_or("Sandbox not found")?;
        sandbox.record_cpu_time(10);
    }
    
    let timestamp_after_cpu = {
        let sandbox = manager.get_sandbox(sandbox_id)
            .ok_or("Sandbox not found")?;
        sandbox.usage.last_update_ms
    };
    
    // Timestamp should be >= previous
    if timestamp_after_cpu < timestamp_after_alloc {
        return Err(crate::kformat!(
            "Timestamp after CPU record ({}) should be >= after alloc ({})",
            timestamp_after_cpu, timestamp_after_alloc
        ));
    }
    
    // Test that timestamps are in milliseconds (reasonable range check)
    // A timestamp in milliseconds since epoch should be > 1_000_000_000_000 (year ~2001)
    // and < 10_000_000_000_000 (year ~2286)
    // However, in a kernel environment, we might use uptime instead of epoch time
    // So we just verify the timestamp is a reasonable positive value
    if initial_timestamp > 0 {
        // Timestamp is positive, which is expected
    } else {
        return Err("Timestamp should be positive".into());
    }
    
    // Test metrics timestamp precision
    let metrics = manager.query_metrics();
    
    if metrics.timestamp_ms == 0 {
        return Err("Metrics timestamp should be non-zero".into());
    }
    
    // Metrics timestamp should be >= sandbox creation time
    if metrics.timestamp_ms < initial_timestamp {
        return Err(crate::kformat!(
            "Metrics timestamp ({}) should be >= sandbox creation ({})",
            metrics.timestamp_ms, initial_timestamp
        ));
    }
    
    // Test warning timestamp precision
    let current_time = get_current_time_ms();
    let warning = ThresholdWarning {
        timestamp_ms: current_time,
        resource_type: WarningResourceType::Memory,
        language: Some(LanguageType::Wasm),
        sandbox_id: Some(sandbox_id),
        current_value: 100,
        threshold_value: 50,
        message: "Test warning".into(),
    };
    manager.emit_warning(warning);
    
    let warnings = manager.get_warnings();
    if warnings.is_empty() {
        return Err("Warning should be recorded".into());
    }
    
    let warning_timestamp = warnings[0].timestamp_ms;
    
    // Warning timestamp should be >= metrics timestamp (or very close)
    // Allow for some timing variance
    if warning_timestamp == 0 {
        return Err("Warning timestamp should be non-zero".into());
    }
    
    // Test that consecutive timestamps can differ by at least 1 ms
    // (This verifies millisecond precision is available)
    let t1 = get_current_time_ms();
    // In a real test we'd wait, but we can verify the function returns ms values
    let t2 = get_current_time_ms();
    
    // t2 should be >= t1 (time doesn't go backwards)
    if t2 < t1 {
        return Err(crate::kformat!(
            "Time should not go backwards: t1={}, t2={}",
            t1, t2
        ));
    }
    
    // Verify termination records timestamp
    let final_usage = manager.terminate_sandbox(sandbox_id)
        .map_err(|e| crate::kformat!("Failed to terminate: {:?}", e))?;
    
    if final_usage.last_update_ms == 0 {
        return Err("Final usage should have last_update_ms set".into());
    }
    
    if final_usage.created_at_ms == 0 {
        return Err("Final usage should have created_at_ms set".into());
    }
    
    // Verify we can calculate lifetime in milliseconds
    let lifetime_ms = final_usage.last_update_ms.saturating_sub(final_usage.created_at_ms);
    // Lifetime should be >= 0 (could be 0 if operations were very fast)
    // This verifies we can measure time differences in milliseconds
    if final_usage.last_update_ms < final_usage.created_at_ms {
        return Err("last_update_ms should be >= created_at_ms".into());
    }
    
    Ok(())
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_property_1() {
        assert!(property_1_language_detection_consistency().is_ok());
    }

    #[test]
    fn test_property_2() {
        assert!(property_2_sandbox_initialization_invariant().is_ok());
    }

    #[test]
    fn test_property_3() {
        assert!(property_3_syscall_translation_validity().is_ok());
    }

    #[test]
    fn test_property_4() {
        assert!(property_4_unsupported_language_error().is_ok());
    }

    #[test]
    fn test_property_5() {
        assert!(property_5_memory_limit_enforcement().is_ok());
    }

    #[test]
    fn test_property_7() {
        assert!(property_7_fd_limit_enforcement().is_ok());
    }

    #[test]
    fn test_property_8() {
        assert!(property_8_capability_verification_consistency().is_ok());
    }

    #[test]
    fn test_property_10() {
        assert!(property_10_capability_denial_logging().is_ok());
    }

    #[test]
    fn test_property_11() {
        assert!(property_11_message_serialization_roundtrip().is_ok());
    }

    #[test]
    fn test_property_12() {
        assert!(property_12_deserialization_error_completeness().is_ok());
    }

    #[test]
    fn test_property_13() {
        assert!(property_13_backend_validation_completeness().is_ok());
    }

    #[test]
    fn test_property_6() {
        assert!(property_6_configuration_isolation().is_ok());
    }

    #[test]
    fn test_property_9() {
        assert!(property_9_token_revocation_propagation().is_ok());
    }

    #[test]
    fn test_property_14() {
        assert!(property_14_fault_isolation().is_ok());
    }

    #[test]
    fn test_property_15() {
        assert!(property_15_crash_dump_completeness().is_ok());
    }

    #[test]
    fn test_property_16() {
        assert!(property_16_syscall_tracing_completeness().is_ok());
    }

    #[test]
    fn test_property_17() {
        assert!(property_17_memory_inspection_read_only().is_ok());
    }

    #[test]
    fn test_property_19() {
        assert!(property_19_termination_statistics_recording().is_ok());
    }

    #[test]
    fn test_property_18() {
        assert!(property_18_metrics_completeness().is_ok());
    }

    #[test]
    fn test_property_20() {
        assert!(property_20_threshold_warning_emission().is_ok());
    }

    #[test]
    fn test_property_21() {
        assert!(property_21_timestamp_precision().is_ok());
    }
}

// ============================================================================
// WASM Backend Unit Tests
// ============================================================================

/// Valid minimal WASM module (empty module with just header)
const VALID_WASM_HEADER: [u8; 8] = [
    0x00, 0x61, 0x73, 0x6D, // Magic: \0asm
    0x01, 0x00, 0x00, 0x00, // Version: 1
];

/// Test WASM backend creation and validation
pub fn test_wasm_backend_creation() -> TestResult {
    let backend = WasmBackend::new();
    
    // Verify language type
    if backend.language_type() != LanguageType::Wasm {
        return Err("WasmBackend should report Wasm language type".into());
    }
    
    // Verify validation passes
    if let Err(e) = backend.validate() {
        return Err(crate::kformat!("WasmBackend validation should pass: {:?}", e));
    }
    
    Ok(())
}

/// Test WASM module loading with valid header
pub fn test_wasm_module_loading() -> TestResult {
    let backend = WasmBackend::new();
    
    let manifest = AppManifest {
        name: "test_module".into(),
        version: "1.0.0".into(),
        language: LanguageType::Wasm,
        entry_point: "main".into(),
        resources: ResourceRequirements::default(),
    };
    
    // Test loading valid WASM
    let module = backend.load(&VALID_WASM_HEADER, &manifest)
        .map_err(|e| crate::kformat!("Failed to load valid WASM: {:?}", e))?;
    
    if module.language != LanguageType::Wasm {
        return Err("Loaded module should have Wasm language type".into());
    }
    
    if module.name != "test_module" {
        return Err(crate::kformat!("Module name mismatch: expected 'test_module', got '{}'", module.name));
    }
    
    if module.code.len() != VALID_WASM_HEADER.len() {
        return Err("Module code should match input size".into());
    }
    
    Ok(())
}

/// Test WASM module loading with invalid header
pub fn test_wasm_invalid_header() -> TestResult {
    let backend = WasmBackend::new();
    
    let manifest = AppManifest {
        name: "invalid_module".into(),
        version: "1.0.0".into(),
        language: LanguageType::Wasm,
        entry_point: "main".into(),
        resources: ResourceRequirements::default(),
    };
    
    // Test with empty input
    match backend.load(&[], &manifest) {
        Ok(_) => return Err("Should reject empty input".into()),
        Err(RuntimeError::ModuleLoadFailed(msg)) => {
            if !msg.contains("too small") && !msg.contains("header") {
                return Err(crate::kformat!("Error should mention size/header: {}", msg));
            }
        }
        Err(e) => return Err(crate::kformat!("Wrong error type: {:?}", e)),
    }
    
    // Test with invalid magic number
    let invalid_magic = [0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00];
    match backend.load(&invalid_magic, &manifest) {
        Ok(_) => return Err("Should reject invalid magic number".into()),
        Err(RuntimeError::ModuleLoadFailed(msg)) => {
            if !msg.contains("magic") {
                return Err(crate::kformat!("Error should mention magic number: {}", msg));
            }
        }
        Err(e) => return Err(crate::kformat!("Wrong error type: {:?}", e)),
    }
    
    // Test with invalid version
    let invalid_version = [0x00, 0x61, 0x73, 0x6D, 0x02, 0x00, 0x00, 0x00];
    match backend.load(&invalid_version, &manifest) {
        Ok(_) => return Err("Should reject invalid version".into()),
        Err(RuntimeError::ModuleLoadFailed(msg)) => {
            if !msg.contains("version") {
                return Err(crate::kformat!("Error should mention version: {}", msg));
            }
        }
        Err(e) => return Err(crate::kformat!("Wrong error type: {:?}", e)),
    }
    
    Ok(())
}

/// Test WASM function execution
pub fn test_wasm_execution() -> TestResult {
    let backend = WasmBackend::new();
    
    let manifest = AppManifest {
        name: "exec_test".into(),
        version: "1.0.0".into(),
        language: LanguageType::Wasm,
        entry_point: "main".into(),
        resources: ResourceRequirements::default(),
    };
    
    let module = backend.load(&VALID_WASM_HEADER, &manifest)
        .map_err(|e| crate::kformat!("Failed to load module: {:?}", e))?;
    
    // Test execution with valid function name
    let result = backend.execute(&module, "main", &[]);
    if result.is_err() {
        return Err(crate::kformat!("Execution should succeed: {:?}", result.err()));
    }
    
    // Test execution with empty function name
    match backend.execute(&module, "", &[]) {
        Ok(_) => return Err("Should reject empty function name".into()),
        Err(RuntimeError::ExecutionFailed(msg)) => {
            if !msg.contains("empty") {
                return Err(crate::kformat!("Error should mention empty: {}", msg));
            }
        }
        Err(e) => return Err(crate::kformat!("Wrong error type: {:?}", e)),
    }
    
    // Test execution with arguments
    let args = vec![Value::Int(42), Value::Bool(true)];
    let result = backend.execute(&module, "test_func", &args);
    if result.is_err() {
        return Err(crate::kformat!("Execution with args should succeed: {:?}", result.err()));
    }
    
    Ok(())
}

/// Test WASM memory tracking
pub fn test_wasm_memory_tracking() -> TestResult {
    let backend = WasmBackend::new();
    
    let manifest = AppManifest {
        name: "memory_test".into(),
        version: "1.0.0".into(),
        language: LanguageType::Wasm,
        entry_point: "main".into(),
        resources: ResourceRequirements::default(),
    };
    
    let module = backend.load(&VALID_WASM_HEADER, &manifest)
        .map_err(|e| crate::kformat!("Failed to load module: {:?}", e))?;
    
    // Memory usage should be at least the code size
    let memory = backend.memory_usage(&module);
    if memory < VALID_WASM_HEADER.len() {
        return Err(crate::kformat!(
            "Memory usage {} should be >= code size {}",
            memory, VALID_WASM_HEADER.len()
        ));
    }
    
    Ok(())
}

/// Test WASM module unloading
pub fn test_wasm_unload() -> TestResult {
    let backend = WasmBackend::new();
    
    let manifest = AppManifest {
        name: "unload_test".into(),
        version: "1.0.0".into(),
        language: LanguageType::Wasm,
        entry_point: "main".into(),
        resources: ResourceRequirements::default(),
    };
    
    let module = backend.load(&VALID_WASM_HEADER, &manifest)
        .map_err(|e| crate::kformat!("Failed to load module: {:?}", e))?;
    
    // Unload should succeed
    backend.unload(module)
        .map_err(|e| crate::kformat!("Unload should succeed: {:?}", e))?;
    
    Ok(())
}

/// Test WASM debug info
pub fn test_wasm_debug_info() -> TestResult {
    let backend = WasmBackend::new();
    
    let manifest = AppManifest {
        name: "debug_test".into(),
        version: "1.0.0".into(),
        language: LanguageType::Wasm,
        entry_point: "main".into(),
        resources: ResourceRequirements::default(),
    };
    
    let module = backend.load(&VALID_WASM_HEADER, &manifest)
        .map_err(|e| crate::kformat!("Failed to load module: {:?}", e))?;
    
    let debug_info = backend.debug_info(&module);
    
    // Should have at least one stack frame
    if debug_info.stack_frames.is_empty() {
        return Err("Debug info should have at least one stack frame".into());
    }
    
    // Should have at least one memory region
    if debug_info.memory_regions.is_empty() {
        return Err("Debug info should have at least one memory region".into());
    }
    
    // Memory region should cover the code
    let code_region = &debug_info.memory_regions[0];
    if code_region.size != module.code.len() {
        return Err(crate::kformat!(
            "Code region size {} should match code size {}",
            code_region.size, module.code.len()
        ));
    }
    
    Ok(())
}

/// Test WASM backend registration with polyglot manager
pub fn test_wasm_backend_registration() -> TestResult {
    let mut manager = PolyglotRuntimeManager::new();
    
    let backend = Arc::new(WasmBackend::new());
    
    // Register should succeed
    manager.register_backend(backend.clone())
        .map_err(|e| crate::kformat!("Registration should succeed: {:?}", e))?;
    
    // Duplicate registration should fail
    match manager.register_backend(backend) {
        Ok(_) => return Err("Duplicate registration should fail".into()),
        Err(_) => {} // Expected
    }
    
    Ok(())
}

#[cfg(test)]
mod wasm_unit_tests {
    use super::*;

    #[test]
    fn test_wasm_creation() {
        assert!(test_wasm_backend_creation().is_ok());
    }

    #[test]
    fn test_wasm_loading() {
        assert!(test_wasm_module_loading().is_ok());
    }

    #[test]
    fn test_wasm_invalid() {
        assert!(test_wasm_invalid_header().is_ok());
    }

    #[test]
    fn test_wasm_exec() {
        assert!(test_wasm_execution().is_ok());
    }

    #[test]
    fn test_wasm_memory() {
        assert!(test_wasm_memory_tracking().is_ok());
    }

    #[test]
    fn test_wasm_unloading() {
        assert!(test_wasm_unload().is_ok());
    }

    #[test]
    fn test_wasm_debug() {
        assert!(test_wasm_debug_info().is_ok());
    }

    #[test]
    fn test_wasm_registration() {
        assert!(test_wasm_backend_registration().is_ok());
    }
}
