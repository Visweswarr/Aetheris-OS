/// ABI Generator for Polymera OS
/// 
/// This tool reads the syscall schema from `abi/syscalls.yaml` and generates:
/// - Kernel dispatch table and handler skeletons
/// - Userland stub functions
/// - C header for external toolchains
/// - Markdown documentation
/// - Examples for C and Rust
/// - ABI linting for backward compatibility

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use serde_yaml;

/// Syscall argument definition
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SyscallArg {
    name: String,
    #[serde(rename = "type")]
    arg_type: String,
    description: String,
}

/// Syscall definition
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Syscall {
    id: u64,
    name: String,
    description: String,
    args: Vec<SyscallArg>,
    return_type: String,
    return_description: String,
    error_codes: Vec<String>,
    implemented: bool,
    category: String,
}

/// Category definition
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Category {
    name: String,
    description: String,
}

/// Error code definition
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ErrorCode {
    name: String,
    value: i32,
}

/// Complete ABI schema
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AbiSchema {
    schema_version: String,
    last_updated: String,
    error_codes: HashMap<String, i32>,
    syscalls: Vec<Syscall>,
    categories: HashMap<String, Category>,
    abi_version: AbiVersion,
}

/// ABI version information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AbiVersion {
    major: u32,
    minor: u32,
    patch: u32,
    stability: String,
    breaking_changes: bool,
}

/// ABI Generator
pub struct AbiGenerator {
    schema: AbiSchema,
    output_dir: String,
}

impl AbiGenerator {
    /// Create a new ABI generator
    pub fn new(schema_path: &str, output_dir: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let schema_content = fs::read_to_string(schema_path)?;
        let schema: AbiSchema = serde_yaml::from_str(&schema_content)?;
        
        Ok(Self {
            schema,
            output_dir: output_dir.to_string(),
        })
    }
    
    /// Generate all ABI artifacts
    pub fn generate_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Generating ABI artifacts...");
        
        // Generate kernel dispatch table
        self.generate_kernel_table()?;
        
        // Generate kernel handlers skeleton
        self.generate_kernel_handlers()?;
        
        // Generate userland stubs
        self.generate_userland_stubs()?;
        
        // Generate C header
        self.generate_c_header()?;
        
        // Generate documentation
        self.generate_documentation()?;
        
        // Generate examples
        self.generate_examples()?;
        
        // Generate schema hash file
        self.generate_schema_hash()?;
        
        println!("✅ All ABI artifacts generated successfully!");
        Ok(())
    }
    
    /// Generate kernel dispatch table
    fn generate_kernel_table(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("  Generating kernel dispatch table...");
        
        let mut content = String::new();
        
        // Header
        content.push_str("/// Auto-generated System Call Number Table for Polymera OS\n");
        content.push_str("/// Generated from abi/syscalls.yaml\n");
        content.push_str("/// DO NOT EDIT MANUALLY - Run tooling/abi/gen.rs to regenerate\n\n");
        
        // Constants
        for syscall in &self.schema.syscalls {
            let const_name = format!("SYS_{}", syscall.name.to_uppercase());
            content.push_str(&format!("/// System call: {}\n", syscall.description));
            content.push_str(&format!("pub const {}: u64 = {};\n\n", const_name, syscall.id));
        }
        
        // Max syscall number
        let max_id = self.schema.syscalls.iter().map(|s| s.id).max().unwrap_or(0);
        content.push_str(&format!("/// Maximum system call number (for validation)\n"));
        content.push_str(&format!("pub const SYS_MAX: u64 = {};\n\n", max_id));
        
        // Syscall info structure
        content.push_str("/// System call information structure\n");
        content.push_str("#[derive(Debug, Clone, Copy)]\n");
        content.push_str("pub struct SyscallInfo {\n");
        content.push_str("    /// System call number\n");
        content.push_str("    pub number: u64,\n");
        content.push_str("    /// System call name\n");
        content.push_str("    pub name: &'static str,\n");
        content.push_str("    /// Number of arguments\n");
        content.push_str("    pub arg_count: u8,\n");
        content.push_str("    /// Whether the syscall is implemented\n");
        content.push_str("    pub implemented: bool,\n");
        content.push_str("    /// Brief description\n");
        content.push_str("    pub description: &'static str,\n");
        content.push_str("    /// Category\n");
        content.push_str("    pub category: &'static str,\n");
        content.push_str("}\n\n");
        
        // Syscall table
        content.push_str("/// System call table with metadata\n");
        content.push_str("pub const SYSCALL_TABLE: &[SyscallInfo] = &[\n");
        
        for syscall in &self.schema.syscalls {
            let category_name = &self.schema.categories.get(&syscall.category)
                .map(|c| c.name.clone())
                .unwrap_or_else(|| syscall.category.clone());
            
            content.push_str(&format!("    SyscallInfo {{\n"));
            content.push_str(&format!("        number: {},\n", syscall.id));
            content.push_str(&format!("        name: \"{}\",\n", syscall.name));
            content.push_str(&format!("        arg_count: {},\n", syscall.args.len()));
            content.push_str(&format!("        implemented: {},\n", syscall.implemented));
            content.push_str(&format!("        description: \"{}\",\n", syscall.description));
            content.push_str(&format!("        category: \"{}\",\n", category_name));
            content.push_str(&format!("    }},\n"));
        }
        
        content.push_str("];\n\n");
        
        // Helper functions
        content.push_str("/// Check if a syscall number is valid\n");
        content.push_str("pub fn is_valid_syscall(num: u64) -> bool {\n");
        content.push_str(&format!("    num > 0 && num <= {}\n", max_id));
        content.push_str("}\n\n");
        
        content.push_str("/// Check if a syscall is implemented\n");
        content.push_str("pub fn is_syscall_implemented(num: u64) -> bool {\n");
        content.push_str("    SYSCALL_TABLE.iter().any(|s| s.number == num && s.implemented)\n");
        content.push_str("}\n\n");
        
        content.push_str("/// Get syscall name by number\n");
        content.push_str("pub fn get_syscall_name(num: u64) -> &'static str {\n");
        content.push_str("    SYSCALL_TABLE.iter()\n");
        content.push_str("        .find(|s| s.number == num)\n");
        content.push_str("        .map(|s| s.name)\n");
        content.push_str("        .unwrap_or(\"unknown\")\n");
        content.push_str("}\n\n");
        
        content.push_str("/// Get syscall info by number\n");
        content.push_str("pub fn get_syscall_info(num: u64) -> Option<&'static SyscallInfo> {\n");
        content.push_str("    SYSCALL_TABLE.iter().find(|s| s.number == num)\n");
        content.push_str("}\n\n");
        
        // Write to file
        let output_path = format!("{}/kernel/src/syscall/table.rs", self.output_dir);
        
        // Create directory if it doesn't exist
        if let Some(parent) = Path::new(&output_path).parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(output_path, content)?;
        
        println!("    ✅ Kernel dispatch table generated");
        Ok(())
    }
    
    /// Generate kernel handlers skeleton
    fn generate_kernel_handlers(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("  Generating kernel handlers skeleton...");
        
        let mut content = String::new();
        
        // Header
        content.push_str("/// Auto-generated System Call Handler Skeletons for Polymera OS\n");
        content.push_str("/// Generated from abi/syscalls.yaml\n");
        content.push_str("/// DO NOT EDIT MANUALLY - Run tooling/abi/gen.rs to regenerate\n\n");
        
        content.push_str("use crate::syscall::table::*;\n\n");
        
        // Handler functions
        for syscall in &self.schema.syscalls {
            content.push_str(&format!("/// Handle {} system call\n", syscall.description));
            content.push_str(&format!("/// \n"));
            content.push_str(&format!("/// # Arguments\n"));
            
            for arg in &syscall.args {
                content.push_str(&format!("/// * `{}` - {}\n", arg.name, arg.description));
            }
            
            content.push_str(&format!("/// \n"));
            content.push_str(&format!("/// # Returns\n"));
            content.push_str(&format!("/// {}\n", syscall.return_description));
            content.push_str(&format!("pub fn handle_{}(_a0: u64, _a1: u64, _a2: u64, _a3: u64) -> u64 {{\n", syscall.name));
            content.push_str(&format!("    // TODO: Implement {}\n", syscall.description));
            content.push_str(&format!("    klog!(TRACE, \"[SYSCALL] Handling {} syscall\");\n", syscall.name));
            content.push_str(&format!("    \n"));
            content.push_str(&format!("    // Return appropriate value based on return type\n"));
            
            match syscall.return_type.as_str() {
                "!" => content.push_str(&format!("    // This function should never return\n")),
                "u64" => content.push_str(&format!("    0 // Success\n")),
                "i32" => content.push_str(&format!("    0 // Success\n")),
                _ => content.push_str(&format!("    // TODO: Implement proper return value\n")),
            }
            
            content.push_str(&format!("}}\n\n"));
        }
        
        // Write to file
        let output_path = format!("{}/kernel/src/syscall/handlers.rs", self.output_dir);
        fs::write(output_path, content)?;
        
        println!("    ✅ Kernel handlers skeleton generated");
        Ok(())
    }
    
    /// Generate userland stubs
    fn generate_userland_stubs(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("  Generating userland stubs...");
        
        let mut content = String::new();
        
        // Header
        content.push_str("/// Auto-generated Userland System Call Stubs for Polymera OS\n");
        content.push_str("/// Generated from abi/syscalls.yaml\n");
        content.push_str("/// DO NOT EDIT MANUALLY - Run tooling/abi/gen.rs to regenerate\n\n");
        
        content.push_str("use std::error::Error;\n\n");
        
        // Stub functions
        for syscall in &self.schema.syscalls {
            content.push_str(&format!("/// {}\n", syscall.description));
            content.push_str(&format!("/// \n"));
            content.push_str(&format!("/// # Arguments\n"));
            
            for arg in &syscall.args {
                content.push_str(&format!("/// * `{}` - {}\n", arg.name, arg.description));
            }
            
            content.push_str(&format!("/// \n"));
            content.push_str(&format!("/// # Returns\n"));
            content.push_str(&format!("/// {}\n", syscall.return_description));
            content.push_str(&format!("pub fn syscall_{}(", syscall.name));
            
            // Function arguments
            let mut args = Vec::new();
            for arg in &syscall.args {
                let arg_type = self.convert_rust_type(&arg.arg_type);
                args.push(format!("{}: {}", arg.name, arg_type));
            }
            content.push_str(&args.join(", "));
            
            content.push_str(&format!(") -> {} {{\n", syscall.return_type));
            content.push_str(&format!("    // TODO: Implement actual syscall invocation\n"));
            content.push_str(&format!("    // This is a stub that should be replaced with real implementation\n"));
            content.push_str(&format!("    \n"));
            content.push_str(&format!("    // For now, return a placeholder value\n"));
            
            match syscall.return_type.as_str() {
                "!" => content.push_str(&format!("    panic!(\"syscall_{} not implemented\");\n", syscall.name)),
                "u64" => content.push_str(&format!("    0 // Placeholder\n")),
                "i32" => content.push_str(&format!("    0 // Placeholder\n")),
                _ => content.push_str(&format!("    // TODO: Implement proper return value\n")),
            }
            
            content.push_str(&format!("}}\n\n"));
        }
        
        // Write to file
        let output_path = format!("{}/userland-stubs/src/lib.rs", self.output_dir);
        fs::write(output_path, content)?;
        
        Ok(())
    }
    
    /// Generate C header
    fn generate_c_header(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("  Generating C header...");
        
        let mut content = String::new();
        
        // Header
        content.push_str("/* Auto-generated C header for Polymera OS syscalls */\n");
        content.push_str("/* Generated from abi/syscalls.yaml */\n");
        content.push_str("/* DO NOT EDIT MANUALLY - Run tooling/abi/gen.rs to regenerate */\n\n");
        
        content.push_str("#ifndef POLYMERA_SYSCALLS_H\n");
        content.push_str("#define POLYMERA_SYSCALLS_H\n\n");
        
        content.push_str("#include <stdint.h>\n");
        content.push_str("#include <stddef.h>\n\n");
        
        // Error codes
        content.push_str("/* Error codes */\n");
        for (name, value) in &self.schema.error_codes {
            content.push_str(&format!("#define {} {}\n", name, value));
        }
        content.push_str("\n");
        
        // Syscall numbers
        content.push_str("/* System call numbers */\n");
        for syscall in &self.schema.syscalls {
            let const_name = format!("SYS_{}", syscall.name.to_uppercase());
            content.push_str(&format!("#define {} {}\n", const_name, syscall.id));
        }
        content.push_str("\n");
        
        // Function declarations
        content.push_str("/* System call function declarations */\n");
        for syscall in &self.schema.syscalls {
            if syscall.implemented {
                content.push_str(&format!("/* {} */\n", syscall.description));
                
                // Function signature
                let return_type = self.convert_c_type(&syscall.return_type);
                content.push_str(&format!("{} sys_{}(", return_type, syscall.name));
                
                // Arguments
                let mut args = Vec::new();
                for arg in &syscall.args {
                    let arg_type = self.convert_c_type(&arg.arg_type);
                    args.push(format!("{} {}", arg_type, arg.name));
                }
                content.push_str(&args.join(", "));
                
                content.push_str(");\n\n");
            }
        }
        
        content.push_str("#endif /* POLYMERA_SYSCALLS_H */\n");
        
        // Write to file
        let output_path = format!("{}/include/polymera_syscalls.h", self.output_dir);
        
        // Create directory if it doesn't exist
        if let Some(parent) = Path::new(&output_path).parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(output_path, content)?;
        
        Ok(())
    }
    
    /// Generate documentation
    fn generate_documentation(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("  Generating documentation...");
        
        let mut content = String::new();
        
        // Header
        content.push_str("# Polymera OS System Calls\n\n");
        content.push_str("This document describes all system calls available in Polymera OS.\n");
        content.push_str("Generated from `abi/syscalls.yaml` - DO NOT EDIT MANUALLY.\n\n");
        
        content.push_str(&format!("**Schema Version:** {}\n", self.schema.schema_version));
        content.push_str(&format!("**Last Updated:** {}\n", self.schema.last_updated));
        content.push_str(&format!("**ABI Version:** {}.{}.{}\n", 
            self.schema.abi_version.major, 
            self.schema.abi_version.minor, 
            self.schema.abi_version.patch));
        content.push_str(&format!("**Stability:** {}\n", self.schema.abi_version.stability));
        content.push_str("\n");
        
        // Categories
        content.push_str("## Categories\n\n");
        for (category_name, category) in &self.schema.categories {
            content.push_str(&format!("### {}\n", category_name));
            content.push_str(&format!("{}\n\n", category.description));
        }
        
        // Syscalls by category
        for (category_name, category) in &self.schema.categories {
            content.push_str(&format!("## {}\n\n", category.name));
            content.push_str(&format!("{}\n\n", category.description));
            
            let category_syscalls: Vec<&Syscall> = self.schema.syscalls.iter()
                .filter(|s| s.category == *category_name)
                .collect();
            
            for syscall in category_syscalls {
                content.push_str(&format!("### {}\n\n", syscall.name));
                content.push_str(&format!("{}\n\n", syscall.description));
                
                // Arguments
                if !syscall.args.is_empty() {
                    content.push_str("**Arguments:**\n\n");
                    for arg in &syscall.args {
                        content.push_str(&format!("- `{}` (`{}`) - {}\n", 
                            arg.name, arg.arg_type, arg.description));
                    }
                    content.push_str("\n");
                }
                
                // Return value
                content.push_str(&format!("**Returns:** `{}` - {}\n\n", 
                    syscall.return_type, syscall.return_description));
                
                // Error codes
                if !syscall.error_codes.is_empty() {
                    content.push_str("**Error Codes:**\n\n");
                    for error_code in &syscall.error_codes {
                        content.push_str(&format!("- `{}`\n", error_code));
                    }
                    content.push_str("\n");
                }
                
                // Implementation status
                let status = if syscall.implemented { "✅ Implemented" } else { "❌ Not Implemented" };
                content.push_str(&format!("**Status:** {}\n\n", status));
            }
        }
        
        // Write to file
        let output_path = format!("{}/docs/abi/SYSCALLS.md", self.output_dir);
        
        // Create directory if it doesn't exist
        if let Some(parent) = Path::new(&output_path).parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(output_path, content)?;
        
        Ok(())
    }
    
    /// Generate examples for C and Rust
    fn generate_examples(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("  Generating examples...");
        
        // Generate Rust examples
        self.generate_rust_examples()?;
        
        // Generate C examples
        self.generate_c_examples()?;
        
        // Generate examples documentation
        self.generate_examples_docs()?;
        
        println!("    ✅ Examples generated");
        Ok(())
    }
    
    /// Generate Rust examples
    fn generate_rust_examples(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut content = String::new();
        
        content.push_str("//! Auto-generated Rust Examples for Polymera OS Syscalls\n");
        content.push_str("//! Generated from abi/syscalls.yaml\n");
        content.push_str("//! DO NOT EDIT MANUALLY - Run tooling/abi/gen.rs to regenerate\n\n");
        
        content.push_str("use std::error::Error;\n\n");
        
        // Generate examples for each syscall
        for syscall in &self.schema.syscalls {
            content.push_str(&self.generate_rust_syscall_example(syscall));
        }
        
        // Generate main function
        content.push_str(&self.generate_rust_main_example());
        
        let output_path = format!("{}/examples/rust_examples.rs", self.output_dir);
        fs::create_dir_all(format!("{}/examples", self.output_dir))?;
        fs::write(output_path, content)?;
        
        Ok(())
    }
    
    /// Generate C examples
    fn generate_c_examples(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut content = String::new();
        
        content.push_str("/* Auto-generated C Examples for Polymera OS Syscalls\n");
        content.push_str(" * Generated from abi/syscalls.yaml\n");
        content.push_str(" * DO NOT EDIT MANUALLY - Run tooling/abi/gen.rs to regenerate\n");
        content.push_str(" */\n\n");
        
        content.push_str("#include <stdio.h>\n");
        content.push_str("#include <stdlib.h>\n");
        content.push_str("#include <string.h>\n");
        content.push_str("#include <unistd.h>\n\n");
        
        // Example buffer for IPC operations
        content.push_str("// Example buffer for IPC operations\n");
        content.push_str("static uint8_t buffer[1024];\n\n");
        
        // Generate examples for each syscall
        for syscall in &self.schema.syscalls {
            content.push_str(&self.generate_c_syscall_example(syscall));
        }
        
        // Generate main function
        content.push_str(&self.generate_c_main_example());
        
        let output_path = format!("{}/examples/c_examples.c", self.output_dir);
        fs::create_dir_all(format!("{}/examples", self.output_dir))?;
        fs::write(output_path, content)?;
        
        Ok(())
    }
    
    /// Generate examples documentation
    fn generate_examples_docs(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut content = String::new();
        
        content.push_str("# Polymera OS Syscall Examples\n\n");
        content.push_str("This document contains runnable examples for all system calls in Polymera OS.\n");
        content.push_str("Examples are generated automatically from the ABI schema.\n\n");
        
        content.push_str("## Rust Examples\n\n");
        for syscall in &self.schema.syscalls {
            content.push_str(&self.generate_rust_example_doc(syscall));
        }
        
        content.push_str("\n## C Examples\n\n");
        for syscall in &self.schema.syscalls {
            content.push_str(&self.generate_c_example_doc(syscall));
        }
        
        let output_path = format!("{}/examples/EXAMPLES.md", self.output_dir);
        fs::write(output_path, content)?;
        
        Ok(())
    }
    
    /// Generate Rust syscall example
    fn generate_rust_syscall_example(&self, syscall: &Syscall) -> String {
        let mut content = String::new();
        
        content.push_str(&format!("/// Example: {}\n", syscall.description));
        content.push_str(&format!("fn example_{}() -> Result<(), Box<dyn Error>> {{\n", syscall.name));
        
        // Generate valid example
        content.push_str("    // Valid usage example\n");
        content.push_str(&format!("    let result = syscall_{}(", syscall.name));
        
        let args = self.generate_rust_example_args(syscall, true);
        content.push_str(&args);
        content.push_str(");\n");
        
        content.push_str("    println!(\"✅ {}: {:?}\", \"");
        content.push_str(&syscall.name);
        content.push_str("\", result);\n");
        
        // Generate invalid example if applicable
        if !syscall.args.is_empty() {
            content.push_str("\n    // Invalid usage example\n");
            content.push_str(&format!("    let invalid_result = syscall_{}(", syscall.name));
            
            let invalid_args = self.generate_rust_example_args(syscall, false);
            content.push_str(&invalid_args);
            content.push_str(");\n");
            
            content.push_str("    println!(\"❌ {} (invalid): {:?}\", \"");
            content.push_str(&syscall.name);
            content.push_str("\", invalid_result);\n");
        }
        
        content.push_str("    Ok(())\n");
        content.push_str("}\n\n");
        
        content
    }
    
    /// Generate C syscall example
    fn generate_c_syscall_example(&self, syscall: &Syscall) -> String {
        let mut content = String::new();
        
        content.push_str(&format!("/* Example: {} */\n", syscall.description));
        content.push_str(&format!("void example_{}() {{\n", syscall.name));
        
        // Generate valid example
        content.push_str("    // Valid usage example\n");
        content.push_str(&format!("    long result = syscall_{}(", syscall.name));
        
        let args = self.generate_c_example_args(syscall, true);
        content.push_str(&args);
        content.push_str(");\n");
        
        content.push_str(&format!("    printf(\"✅ {}: %ld\\n\", result);\n", syscall.name));
        
        // Generate invalid example if applicable
        if !syscall.args.is_empty() {
            content.push_str("\n    // Invalid usage example\n");
            content.push_str(&format!("    long invalid_result = syscall_{}(", syscall.name));
            
            let invalid_args = self.generate_c_example_args(syscall, false);
            content.push_str(&invalid_args);
            content.push_str(");\n");
            
            content.push_str(&format!("    printf(\"❌ {} (invalid): %ld\\n\", invalid_result);\n", syscall.name));
        }
        
        content.push_str("}\n\n");
        
        content
    }
    
    /// Generate Rust example arguments
    fn generate_rust_example_args(&self, syscall: &Syscall, valid: bool) -> String {
        let mut args = Vec::new();
        
        for arg in &syscall.args {
            let arg_value = if valid {
                self.generate_valid_rust_arg(arg)
            } else {
                self.generate_invalid_rust_arg(arg)
            };
            args.push(arg_value);
        }
        
        args.join(", ")
    }
    
    /// Generate C example arguments
    fn generate_c_example_args(&self, syscall: &Syscall, valid: bool) -> String {
        let mut args = Vec::new();
        
        for arg in &syscall.args {
            let arg_value = if valid {
                self.generate_valid_c_arg(arg)
            } else {
                self.generate_invalid_c_arg(arg)
            };
            args.push(arg_value);
        }
        
        args.join(", ")
    }
    
    /// Generate valid Rust argument value
    fn generate_valid_rust_arg(&self, arg: &SyscallArg) -> String {
        match arg.arg_type.as_str() {
            "u64" => "42u64".to_string(),
            "i32" => "0i32".to_string(),
            "bool" => "true".to_string(),
            "&[u8]" => "&[1, 2, 3, 4]".to_string(),
            "&mut [u8]" => "&mut [0u8; 1024]".to_string(),
            "&str" => "\"example\"".to_string(),
            _ => format!("/* TODO: Implement for type {} */", arg.arg_type),
        }
    }
    
    /// Generate invalid Rust argument value
    fn generate_invalid_rust_arg(&self, arg: &SyscallArg) -> String {
        match arg.arg_type.as_str() {
            "u64" => "u64::MAX".to_string(),
            "i32" => "-1i32".to_string(),
            "bool" => "true".to_string(), // bool doesn't have invalid values
            "&[u8]" => "&[]".to_string(), // empty slice
            "&mut [u8]" => "&mut []".to_string(), // empty slice
            "&str" => "\"\"".to_string(), // empty string
            _ => format!("/* TODO: Implement for type {} */", arg.arg_type),
        }
    }
    
    /// Generate valid C argument value
    fn generate_valid_c_arg(&self, arg: &SyscallArg) -> String {
        match arg.arg_type.as_str() {
            "u64" => "42ULL".to_string(),
            "i32" => "0".to_string(),
            "bool" => "1".to_string(),
            "&[u8]" => "buffer, sizeof(buffer)".to_string(),
            "&mut [u8]" => "buffer, sizeof(buffer)".to_string(),
            "&str" => "\"example\"".to_string(),
            _ => format!("/* TODO: Implement for type %s */", arg.arg_type),
        }
    }
    
    /// Generate invalid C argument value
    fn generate_invalid_c_arg(&self, arg: &SyscallArg) -> String {
        match arg.arg_type.as_str() {
            "u64" => "0xFFFFFFFFFFFFFFFFULL".to_string(),
            "i32" => "-1".to_string(),
            "bool" => "1".to_string(), // bool doesn't have invalid values
            "&[u8]" => "NULL, 0".to_string(), // null pointer
            "&mut [u8]" => "NULL, 0".to_string(), // null pointer
            "&str" => "NULL".to_string(), // null pointer
            _ => format!("/* TODO: Implement for type %s */", arg.arg_type),
        }
    }
    
    /// Generate Rust main example
    fn generate_rust_main_example(&self) -> String {
        let mut content = String::new();
        
        content.push_str("fn main() -> Result<(), Box<dyn Error>> {\n");
        content.push_str("    println!(\"🚀 Running Polymera OS Syscall Examples\");\n\n");
        
        for syscall in &self.schema.syscalls {
            content.push_str(&format!("    example_{}()?;\n", syscall.name));
        }
        
        content.push_str("\n    println!(\"✅ All examples completed successfully!\");\n");
        content.push_str("    Ok(())\n");
        content.push_str("}\n");
        
        content
    }
    
    /// Generate C main example
    fn generate_c_main_example(&self) -> String {
        let mut content = String::new();
        
        content.push_str("int main() {\n");
        content.push_str("    printf(\"🚀 Running Polymera OS Syscall Examples\\n\\n\");\n");
        
        for syscall in &self.schema.syscalls {
            content.push_str(&format!("    example_{}();\n", syscall.name));
        }
        
        content.push_str("\n    printf(\"✅ All examples completed successfully!\\n\");\n");
        content.push_str("    return 0;\n");
        content.push_str("}\n");
        
        content
    }
    
    /// Generate Rust example documentation
    fn generate_rust_example_doc(&self, syscall: &Syscall) -> String {
        let mut content = String::new();
        
        content.push_str(&format!("### {}\n\n", syscall.name));
        content.push_str(&format!("{}\n\n", syscall.description));
        
        content.push_str("```rust\n");
        content.push_str(&format!("// Valid usage\n"));
        content.push_str(&format!("let result = syscall_{}(", syscall.name));
        
        let args = self.generate_rust_example_args(syscall, true);
        content.push_str(&args);
        content.push_str(");\n");
        
        content.push_str("```\n\n");
        
        if !syscall.args.is_empty() {
            content.push_str("```rust\n");
            content.push_str(&format!("// Invalid usage\n"));
            content.push_str(&format!("let result = syscall_{}(", syscall.name));
            
            let invalid_args = self.generate_rust_example_args(syscall, false);
            content.push_str(&invalid_args);
            content.push_str(");\n");
            
            content.push_str("```\n\n");
        }
        
        content
    }
    
    /// Generate C example documentation
    fn generate_c_example_doc(&self, syscall: &Syscall) -> String {
        let mut content = String::new();
        
        content.push_str(&format!("### {}\n\n", syscall.name));
        content.push_str(&format!("{}\n\n", syscall.description));
        
        content.push_str("```c\n");
        content.push_str(&format!("// Valid usage\n"));
        content.push_str(&format!("long result = syscall_{}(", syscall.name));
        
        let args = self.generate_c_example_args(syscall, true);
        content.push_str(&args);
        content.push_str(");\n");
        
        content.push_str("```\n\n");
        
        if !syscall.args.is_empty() {
            content.push_str("```c\n");
            content.push_str(&format!("// Invalid usage\n"));
            content.push_str(&format!("long result = syscall_{}(", syscall.name));
            
            let invalid_args = self.generate_c_example_args(syscall, false);
            content.push_str(&invalid_args);
            content.push_str(");\n");
            
            content.push_str("```\n\n");
        }
        
        content
    }
    
    /// Generate schema hash file
    fn generate_schema_hash(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("  Generating schema hash file...");
        
        let hash = self.calculate_schema_hash();
        let content = format!(
            "/// Auto-generated ABI Schema Hash for Polymera OS\n\
             /// Generated from abi/syscalls.yaml\n\
             /// DO NOT EDIT MANUALLY - Run tooling/abi/gen.rs to regenerate\n\n\
             /// Schema hash for validation\n\
             pub const ABI_SCHEMA_HASH: &str = \"{}\";\n\n\
             /// Schema version\n\
             pub const ABI_SCHEMA_VERSION: &str = \"{}\";\n\n\
             /// Last updated timestamp\n\
             pub const ABI_LAST_UPDATED: &str = \"{}\";\n",
            hash, self.schema.schema_version, self.schema.last_updated
        );
        
        let output_path = format!("{}/schema_hash.rs", self.output_dir);
        fs::write(output_path, content)?;
        
        println!("    ✅ Schema hash file generated");
        Ok(())
    }
    
    /// Calculate schema hash for validation
    fn calculate_schema_hash(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        self.schema.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
    
    /// Convert Rust type to C type
    fn convert_c_type(&self, rust_type: &str) -> String {
        match rust_type {
            "u64" => "uint64_t".to_string(),
            "i32" => "int32_t".to_string(),
            "bool" => "int".to_string(),
            "&[u8]" => "const uint8_t*".to_string(),
            "&mut [u8]" => "uint8_t*".to_string(),
            "&str" => "const char*".to_string(),
            "&[&str]" => "const char**".to_string(),
            "!" => "void".to_string(),
            _ => rust_type.to_string(),
        }
    }
    
    /// Convert Rust type for function signatures
    fn convert_rust_type(&self, rust_type: &str) -> String {
        match rust_type {
            "!" => "!".to_string(),
            _ => rust_type.to_string(),
        }
    }
}

/// Main function for command-line usage
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <schema_path> <output_dir>", args[0]);
        eprintln!("Example: {} abi/syscalls.yaml .", args[0]);
        std::process::exit(1);
    }
    
    let schema_path = &args[1];
    let output_dir = &args[2];
    
    let generator = AbiGenerator::new(schema_path, output_dir)?;
    generator.generate_all()?;
    
    Ok(())
}
