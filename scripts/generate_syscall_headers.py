#!/usr/bin/env python3
"""
System Call Header Generator

This script parses the SYSCALLS.md documentation and generates:
1. Rust header file for kernel use
2. C header file for user space use
3. Assembly constants for syscall numbers

The generated files ensure kernel and user stubs never drift by using
a single source of truth (SYSCALLS.md).
"""

import re
import sys
import os
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass

@dataclass
class SyscallDefinition:
    """Represents a single system call definition"""
    number: int
    name: str
    description: str
    arguments: str
    return_value: str

@dataclass
class ErrorCode:
    """Represents an error code definition"""
    code: int
    name: str
    description: str

@dataclass
class DataType:
    """Represents a data type definition"""
    name: str
    definition: str
    comment: str

class SyscallHeaderGenerator:
    """Generates system call headers from SYSCALLS.md"""
    
    def __init__(self, docs_path: str):
        self.docs_path = Path(docs_path)
        self.syscalls: List[SyscallDefinition] = []
        self.error_codes: List[ErrorCode] = []
        self.data_types: List[DataType] = []
        
    def parse_syscalls_md(self) -> None:
        """Parse the SYSCALLS.md file and extract all definitions"""
        md_file = self.docs_path / "docs" / "abi" / "SYSCALLS.md"
        
        if not md_file.exists():
            raise FileNotFoundError(f"SYSCALLS.md not found at {md_file}")
        
        content = md_file.read_text()
        
        # Parse system call definitions
        self._parse_syscall_tables(content)
        
        # Parse error codes
        self._parse_error_codes(content)
        
        # Parse data types
        self._parse_data_types(content)
        
        print(f"Parsed {len(self.syscalls)} syscalls, {len(self.error_codes)} error codes, {len(self.data_types)} data types")
    
    def _parse_syscall_tables(self, content: str) -> None:
        """Parse system call tables from markdown content"""
        # Find all table sections
        table_pattern = r'### \*\*(.*?)\*\*\s*\n\s*\|.*?\|\n\s*\|.*?\|\n(.*?)(?=###|\Z)'
        tables = re.findall(table_pattern, content, re.DOTALL)
        
        for table_name, table_content in tables:
            if "System Call" in table_name or "Calls" in table_name:
                self._parse_syscall_table(table_content)
    
    def _parse_syscall_table(self, table_content: str) -> None:
        """Parse a single system call table"""
        # Split into lines and skip header separators
        lines = [line.strip() for line in table_content.split('\n') if line.strip() and '|' in line]
        
        for line in lines:
            # Skip separator lines
            if line.startswith('|--'):
                continue
                
            # Parse table row
            parts = [part.strip() for part in line.split('|')[1:-1]]
            if len(parts) >= 5:
                try:
                    number = int(parts[0])
                    name = parts[1].strip('`')
                    description = parts[2]
                    arguments = parts[3]
                    return_value = parts[4]
                    
                    syscall = SyscallDefinition(
                        number=number,
                        name=name,
                        description=description,
                        arguments=arguments,
                        return_value=return_value
                    )
                    self.syscalls.append(syscall)
                except (ValueError, IndexError):
                    continue
    
    def _parse_error_codes(self, content: str) -> None:
        """Parse error code definitions"""
        # Find error code tables
        error_pattern = r'### \*\*Standard Error Numbers\*\*\s*\n\s*\|.*?\|\n\s*\|.*?\|\n(.*?)(?=###|\Z)'
        error_match = re.search(error_pattern, content, re.DOTALL)
        
        if error_match:
            error_table = error_match.group(1)
            lines = [line.strip() for line in error_table.split('\n') if line.strip() and '|' in line]
            
            for line in lines:
                if line.startswith('|--'):
                    continue
                    
                parts = [part.strip() for part in line.split('|')[1:-1]]
                if len(parts) >= 3:
                    try:
                        code = int(parts[0])
                        name = parts[1].strip('`')
                        description = parts[2]
                        
                        error_code = ErrorCode(
                            code=code,
                            name=name,
                            description=description
                        )
                        self.error_codes.append(error_code)
                    except (ValueError, IndexError):
                        continue
        
        # Parse Polymera OS specific errors
        polymera_error_pattern = r'### \*\*Polymera OS Specific Errors\*\*\s*\n\s*\|.*?\|\n\s*\|.*?\|\n(.*?)(?=###|\Z)'
        polymera_match = re.search(polymera_error_pattern, content, re.DOTALL)
        
        if polymera_match:
            polymera_table = polymera_match.group(1)
            lines = [line.strip() for line in polymera_table.split('\n') if line.strip() and '|' in line]
            
            for line in lines:
                if line.startswith('|--'):
                    continue
                    
                parts = [part.strip() for part in line.split('|')[1:-1]]
                if len(parts) >= 3:
                    try:
                        code = int(parts[0])
                        name = parts[1].strip('`')
                        description = parts[2]
                        
                        error_code = ErrorCode(
                            code=code,
                            name=name,
                            description=description
                        )
                        self.error_codes.append(error_code)
                    except (ValueError, IndexError):
                        continue
    
    def _parse_data_types(self, content: str) -> None:
        """Parse data type definitions"""
        # Find data type sections
        type_pattern = r'### \*\*Basic Types\*\*\s*\n```c\n(.*?)\n```'
        type_match = re.search(type_pattern, content, re.DOTALL)
        
        if type_match:
            type_content = type_match.group(1)
            lines = type_content.split('\n')
            
            for line in lines:
                line = line.strip()
                if line and 'typedef' in line:
                    # Parse typedef line
                    match = re.match(r'typedef\s+(\w+)\s+(\w+);\s*//\s*(.*)', line)
                    if match:
                        definition, name, comment = match.groups()
                        data_type = DataType(
                            name=name,
                            definition=definition,
                            comment=comment
                        )
                        self.data_types.append(data_type)
    
    def generate_rust_header(self, output_path: str) -> None:
        """Generate Rust header file for kernel use"""
        output_file = Path(output_path)
        output_file.parent.mkdir(parents=True, exist_ok=True)
        
        with open(output_file, 'w') as f:
            f.write("//! System Call Definitions - Auto-generated from SYSCALLS.md\n")
            f.write("//! Do not edit this file directly. Edit docs/abi/SYSCALLS.md instead.\n\n")
            f.write("#![allow(non_upper_case_globals)]\n")
            f.write("#![allow(dead_code)]\n\n")
            
            # Write data types
            f.write("// Data Types\n")
            for data_type in self.data_types:
                f.write(f"/// {data_type.comment}\n")
                f.write(f"pub type {data_type.name} = {data_type.definition};\n\n")
            
            # Write system call numbers
            f.write("// System Call Numbers\n")
            for syscall in sorted(self.syscalls, key=lambda x: x.number):
                f.write(f"/// {syscall.description}\n")
                f.write(f"/// Arguments: {syscall.arguments}\n")
                f.write(f"/// Returns: {syscall.return_value}\n")
                f.write(f"pub const {syscall.name}: i64 = {syscall.number};\n\n")
            
            # Write error codes
            f.write("// Error Codes\n")
            for error in sorted(self.error_codes, key=lambda x: x.code):
                f.write(f"/// {error.description}\n")
                f.write(f"pub const {error.name}: i64 = {error.code};\n\n")
            
            # Write helper functions
            f.write("// Helper Functions\n")
            f.write("/// Check if a return value indicates an error\n")
            f.write("pub fn is_error(ret: i64) -> bool {\n")
            f.write("    ret < 0\n")
            f.write("}\n\n")
            
            f.write("/// Get error code from return value\n")
            f.write("pub fn get_error_code(ret: i64) -> i64 {\n")
            f.write("    if is_error(ret) {\n")
            f.write("        ret\n")
            f.write("    } else {\n")
            f.write("        0\n")
            f.write("    }\n")
            f.write("}\n\n")
            
            f.write("/// Get success value from return value\n")
            f.write("pub fn get_success_value(ret: i64) -> i64 {\n")
            f.write("    if is_error(ret) {\n")
            f.write("        0\n")
            f.write("    } else {\n")
            f.write("        ret\n")
            f.write("    }\n")
            f.write("}\n")
        
        print(f"Generated Rust header: {output_file}")
    
    def generate_c_header(self, output_path: str) -> None:
        """Generate C header file for user space use"""
        output_file = Path(output_path)
        output_file.parent.mkdir(parents=True, exist_ok=True)
        
        with open(output_file, 'w') as f:
            f.write("/* System Call Definitions - Auto-generated from SYSCALLS.md */\n")
            f.write("/* Do not edit this file directly. Edit docs/abi/SYSCALLS.md instead. */\n\n")
            
            # Write header guards
            f.write("#ifndef POLYMERA_SYSCALLS_H\n")
            f.write("#define POLYMERA_SYSCALLS_H\n\n")
            
            # Write includes
            f.write("#include <stdint.h>\n")
            f.write("#include <sys/types.h>\n\n")
            
            # Write data types
            f.write("/* Data Types */\n")
            for data_type in self.data_types:
                f.write(f"/* {data_type.comment} */\n")
                f.write(f"typedef {data_type.definition} {data_type.name};\n\n")
            
            # Write system call numbers
            f.write("/* System Call Numbers */\n")
            for syscall in sorted(self.syscalls, key=lambda x: x.number):
                f.write(f"/* {syscall.description} */\n")
                f.write(f"/* Arguments: {syscall.arguments} */\n")
                f.write(f"/* Returns: {syscall.return_value} */\n")
                f.write(f"#define {syscall.name} {syscall.number}\n\n")
            
            # Write error codes
            f.write("/* Error Codes */\n")
            for error in sorted(self.error_codes, key=lambda x: x.code):
                f.write(f"/* {error.description} */\n")
                f.write(f"#define {error.name} {error.code}\n\n")
            
            # Write helper functions
            f.write("/* Helper Functions */\n")
            f.write("/* Check if a return value indicates an error */\n")
            f.write("static inline int is_error(int64_t ret) {\n")
            f.write("    return ret < 0;\n")
            f.write("}\n\n")
            
            f.write("/* Get error code from return value */\n")
            f.write("static inline int64_t get_error_code(int64_t ret) {\n")
            f.write("    return is_error(ret) ? ret : 0;\n")
            f.write("}\n\n")
            
            f.write("/* Get success value from return value */\n")
            f.write("static inline int64_t get_success_value(int64_t ret) {\n")
            f.write("    return is_error(ret) ? 0 : ret;\n")
            f.write("}\n\n")
            
            # Write syscall function prototype
            f.write("/* System Call Function */\n")
            f.write("int64_t syscall(int64_t number, ...);\n\n")
            
            # Write footer
            f.write("#endif /* POLYMERA_SYSCALLS_H */\n")
        
        print(f"Generated C header: {output_file}")
    
    def generate_assembly_constants(self, output_path: str) -> None:
        """Generate assembly constants for syscall numbers"""
        output_file = Path(output_path)
        output_file.parent.mkdir(parents=True, exist_ok=True)
        
        with open(output_file, 'w') as f:
            f.write("; System Call Constants - Auto-generated from SYSCALLS.md\n")
            f.write("; Do not edit this file directly. Edit docs/abi/SYSCALLS.md instead.\n\n")
            
            # Write system call numbers
            f.write("; System Call Numbers\n")
            for syscall in sorted(self.syscalls, key=lambda x: x.number):
                f.write(f"; {syscall.description}\n")
                f.write(f"; Arguments: {syscall.arguments}\n")
                f.write(f"; Returns: {syscall.return_value}\n")
                f.write(f"SYS_{syscall.name} equ {syscall.number}\n\n")
            
            # Write error codes
            f.write("; Error Codes\n")
            for error in sorted(self.error_codes, key=lambda x: x.code):
                f.write(f"; {error.description}\n")
                f.write(f"{error.name} equ {error.code}\n\n")
        
        print(f"Generated assembly constants: {output_file}")
    
    def generate_all_headers(self, output_dir: str) -> None:
        """Generate all header files in the specified output directory"""
        output_path = Path(output_dir)
        
        # Generate Rust header for kernel
        rust_header = output_path / "kernel" / "src" / "syscall" / "generated.rs"
        self.generate_rust_header(str(rust_header))
        
        # Generate C header for user space
        c_header = output_path / "userland-stubs" / "include" / "polymera" / "syscalls.h"
        self.generate_c_header(str(c_header))
        
        # Generate assembly constants
        asm_constants = output_path / "kernel" / "src" / "syscall" / "generated.asm"
        self.generate_assembly_constants(str(asm_constants))
        
        # Generate a summary file
        summary_file = output_path / "syscall_generation_summary.txt"
        with open(summary_file, 'w') as f:
            f.write("System Call Header Generation Summary\n")
            f.write("====================================\n\n")
            f.write(f"Generated at: {Path.cwd()}\n")
            f.write(f"Source: {self.docs_path / 'docs' / 'abi' / 'SYSCALLS.md'}\n\n")
            f.write(f"System Calls: {len(self.syscalls)}\n")
            f.write(f"Error Codes: {len(self.error_codes)}\n")
            f.write(f"Data Types: {len(self.data_types)}\n\n")
            f.write("Generated Files:\n")
            f.write(f"  - Rust Header: {rust_header}\n")
            f.write(f"  - C Header: {c_header}\n")
            f.write(f"  - Assembly Constants: {asm_constants}\n")
        
        print(f"Generated summary: {summary_file}")

def main():
    """Main entry point"""
    if len(sys.argv) != 2:
        print("Usage: python3 generate_syscall_headers.py <project_root>")
        sys.exit(1)
    
    project_root = sys.argv[1]
    
    try:
        # Create generator
        generator = SyscallHeaderGenerator(project_root)
        
        # Parse SYSCALLS.md
        generator.parse_syscalls_md()
        
        # Generate all headers
        generator.generate_all_headers(project_root)
        
        print("✅ System call header generation completed successfully!")
        
    except Exception as e:
        print(f"❌ Error generating syscall headers: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
