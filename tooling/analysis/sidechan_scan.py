#!/usr/bin/env python3
"""
Side-Channel Vulnerability Scanner for Polymera OS

This tool performs static analysis on Rust assembly and LLVM-IR to detect
potential side-channel vulnerabilities in PQC wrappers and IPC MAC implementations.

Usage:
    python sidechan_scan.py [--asm-dir DIR] [--llvm-dir DIR] [--output FILE] [--verbose]
"""

import argparse
import os
import re
import sys
import json
from pathlib import Path
from typing import List, Dict, Set, Tuple, Optional
from dataclasses import dataclass, asdict
from enum import Enum
import subprocess
import logging

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

class VulnerabilityType(Enum):
    """Types of side-channel vulnerabilities"""
    SECRET_DEPENDENT_BRANCH = "secret_dependent_branch"
    SECRET_DEPENDENT_TABLE_LOOKUP = "secret_dependent_table_lookup"
    SECRET_DEPENDENT_MEMORY_ACCESS = "secret_dependent_memory_access"
    SECRET_DEPENDENT_LOOP = "secret_dependent_loop"
    SECRET_DEPENDENT_FUNCTION_CALL = "secret_dependent_function_call"
    TIMING_LEAK = "timing_leak"
    CACHE_LEAK = "cache_leak"

class Severity(Enum):
    """Vulnerability severity levels"""
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"

@dataclass
class Vulnerability:
    """Represents a detected vulnerability"""
    file_path: str
    line_number: int
    vulnerability_type: VulnerabilityType
    severity: Severity
    description: str
    code_snippet: str
    context: str
    recommendation: str
    confidence: float  # 0.0 to 1.0

@dataclass
class ScanResult:
    """Results of a side-channel vulnerability scan"""
    total_files_scanned: int
    total_vulnerabilities: int
    vulnerabilities_by_type: Dict[str, int]
    vulnerabilities_by_severity: Dict[str, int]
    vulnerabilities: List[Vulnerability]
    scan_duration: float
    timestamp: str

class SideChannelScanner:
    """Main scanner class for detecting side-channel vulnerabilities"""
    
    def __init__(self, verbose: bool = False):
        self.verbose = verbose
        self.secret_patterns = self._build_secret_patterns()
        self.vulnerability_patterns = self._build_vulnerability_patterns()
        self.whitelist_patterns = self._build_whitelist_patterns()
        
        if verbose:
            logging.getLogger().setLevel(logging.DEBUG)
    
    def _build_secret_patterns(self) -> List[re.Pattern]:
        """Build regex patterns for identifying secret data"""
        patterns = [
            # Secret variable names
            re.compile(r'\b(?:secret|key|password|token|nonce|salt|iv|seed)\w*\b', re.IGNORECASE),
            # Cryptographic function names
            re.compile(r'\b(?:encrypt|decrypt|sign|verify|hash|hmac|mac|kdf|prf)\w*\b', re.IGNORECASE),
            # PQC-specific patterns
            re.compile(r'\b(?:kyber|dilithium|sphincs|falcon|ntru|lattice|quantum)\w*\b', re.IGNORECASE),
            # IPC and MAC patterns
            re.compile(r'\b(?:ipc|mac|message|packet|frame|header)\w*\b', re.IGNORECASE),
        ]
        return patterns
    
    def _build_vulnerability_patterns(self) -> Dict[VulnerabilityType, List[re.Pattern]]:
        """Build regex patterns for detecting specific vulnerability types"""
        patterns = {
            VulnerabilityType.SECRET_DEPENDENT_BRANCH: [
                # Conditional branches based on secret data
                re.compile(r'\b(?:if|while|for)\s*\([^)]*[a-zA-Z_]\w*[^)]*\)', re.IGNORECASE),
                # Switch statements with secret-dependent cases
                re.compile(r'\bswitch\s*\([^)]*[a-zA-Z_]\w*[^)]*\)', re.IGNORECASE),
                # Ternary operators
                re.compile(r'\?[^:]*:[^;]*;', re.IGNORECASE),
            ],
            
            VulnerabilityType.SECRET_DEPENDENT_TABLE_LOOKUP: [
                # Array indexing with secret data
                re.compile(r'\[[^]]*[a-zA-Z_]\w*[^]]*\]', re.IGNORECASE),
                # Table lookups
                re.compile(r'\b(?:table|array|vector|map|hashmap)\s*\[[^]]*[a-zA-Z_]\w*[^]]*\]', re.IGNORECASE),
            ],
            
            VulnerabilityType.SECRET_DEPENDENT_MEMORY_ACCESS: [
                # Memory access patterns
                re.compile(r'\b(?:load|store|read|write|memcpy|memset)\s*\([^)]*[a-zA-Z_]\w*[^)]*\)', re.IGNORECASE),
                # Pointer arithmetic
                re.compile(r'\b(?:ptr|pointer)\s*[+\-]\s*[a-zA-Z_]\w*', re.IGNORECASE),
            ],
            
            VulnerabilityType.SECRET_DEPENDENT_LOOP: [
                # Loops with secret-dependent bounds
                re.compile(r'\b(?:for|while)\s*\([^)]*[a-zA-Z_]\w*[^)]*\)', re.IGNORECASE),
                # Loop counters based on secrets
                re.compile(r'\b(?:count|index|offset|length|size)\s*=\s*[a-zA-Z_]\w*', re.IGNORECASE),
            ],
            
            VulnerabilityType.SECRET_DEPENDENT_FUNCTION_CALL: [
                # Function calls with secret parameters
                re.compile(r'\b(?:call|invoke|execute)\s*\([^)]*[a-zA-Z_]\w*[^)]*\)', re.IGNORECASE),
                # Method calls
                re.compile(r'\.[a-zA-Z_]\w*\s*\([^)]*[a-zA-Z_]\w*[^)]*\)', re.IGNORECASE),
            ],
            
            VulnerabilityType.TIMING_LEAK: [
                # Early returns based on secrets
                re.compile(r'\breturn\s+[^;]*[a-zA-Z_]\w*[^;]*;', re.IGNORECASE),
                # Conditional returns
                re.compile(r'\bif\s*\([^)]*[a-zA-Z_]\w*[^)]*\)\s*return', re.IGNORECASE),
            ],
            
            VulnerabilityType.CACHE_LEAK: [
                # Memory access patterns that could leak cache information
                re.compile(r'\b(?:access|fetch|load)\s*\([^)]*[a-zA-Z_]\w*[^)]*\)', re.IGNORECASE),
                # Array traversal patterns
                re.compile(r'\b(?:traverse|iterate|scan)\s*\([^)]*[a-zA-Z_]\w*[^)]*\)', re.IGNORECASE),
            ],
        }
        return patterns
    
    def _build_whitelist_patterns(self) -> List[re.Pattern]:
        """Build patterns for whitelisted code that should be ignored"""
        patterns = [
            # Test code
            re.compile(r'\b(?:test|spec|example|demo)\w*\b', re.IGNORECASE),
            # Debug/logging code
            re.compile(r'\b(?:debug|log|trace|print|println)\w*\b', re.IGNORECASE),
            # Documentation
            re.compile(r'\b(?:doc|comment|note|todo|fixme)\w*\b', re.IGNORECASE),
            # Safe cryptographic operations
            re.compile(r'\b(?:constant_time|secure|safe|protected)\w*\b', re.IGNORECASE),
        ]
        return patterns
    
    def scan_file(self, file_path: str) -> List[Vulnerability]:
        """Scan a single file for side-channel vulnerabilities"""
        vulnerabilities = []
        
        try:
            with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
                lines = content.split('\n')
        except Exception as e:
            logger.warning(f"Could not read file {file_path}: {e}")
            return vulnerabilities
        
        # Skip whitelisted files
        if self._is_whitelisted(file_path, content):
            if self.verbose:
                logger.debug(f"Skipping whitelisted file: {file_path}")
            return vulnerabilities
        
        # Scan each line for vulnerabilities
        for line_num, line in enumerate(lines, 1):
            line_vulns = self._scan_line(line, line_num, file_path)
            vulnerabilities.extend(line_vulns)
        
        # Scan for multi-line patterns
        multi_line_vulns = self._scan_multiline_patterns(content, file_path)
        vulnerabilities.extend(multi_line_vulns)
        
        return vulnerabilities
    
    def _is_whitelisted(self, file_path: str, content: str) -> bool:
        """Check if a file should be whitelisted"""
        # Check file path
        for pattern in self.whitelist_patterns:
            if pattern.search(file_path):
                return True
        
        # Check content for whitelist markers
        whitelist_markers = [
            '# SIDE_CHANNEL_SAFE',
            '// SIDE_CHANNEL_SAFE',
            '/* SIDE_CHANNEL_SAFE */',
            '# NO_SIDE_CHANNEL_SCAN',
            '// NO_SIDE_CHANNEL_SCAN',
        ]
        
        for marker in whitelist_markers:
            if marker in content:
                return True
        
        return False
    
    def _scan_line(self, line: str, line_num: int, file_path: str) -> List[Vulnerability]:
        """Scan a single line for vulnerabilities"""
        vulnerabilities = []
        
        # Check if line contains secret data
        if not self._contains_secret_data(line):
            return vulnerabilities
        
        # Check for each vulnerability type
        for vuln_type, patterns in self.vulnerability_patterns.items():
            for pattern in patterns:
                if pattern.search(line):
                    vuln = self._create_vulnerability(
                        file_path, line_num, vuln_type, line, pattern
                    )
                    if vuln:
                        vulnerabilities.append(vuln)
        
        return vulnerabilities
    
    def _contains_secret_data(self, line: str) -> bool:
        """Check if a line contains potential secret data"""
        for pattern in self.secret_patterns:
            if pattern.search(line):
                return True
        return False
    
    def _scan_multiline_patterns(self, content: str, file_path: str) -> List[Vulnerability]:
        """Scan for vulnerabilities that span multiple lines"""
        vulnerabilities = []
        
        # Look for function definitions that might contain secret-dependent logic
        function_patterns = [
            # Rust function definitions
            re.compile(r'fn\s+(\w+)\s*\([^)]*\)\s*->?\s*[^{]*\{', re.MULTILINE | re.DOTALL),
            # LLVM function definitions
            re.compile(r'define\s+\w+\s+@(\w+)\s*\([^)]*\)\s*\{', re.MULTILINE | re.DOTALL),
        ]
        
        for pattern in function_patterns:
            for match in pattern.finditer(content):
                func_name = match.group(1)
                func_start = match.start()
                func_end = self._find_function_end(content, func_start)
                
                if func_end > func_start:
                    func_content = content[func_start:func_end]
                    func_vulns = self._analyze_function(func_content, func_name, file_path)
                    vulnerabilities.extend(func_vulns)
        
        return vulnerabilities
    
    def _find_function_end(self, content: str, start_pos: int) -> int:
        """Find the end of a function starting at start_pos"""
        brace_count = 0
        in_string = False
        string_char = None
        
        for i, char in enumerate(content[start_pos:], start_pos):
            if char == '"' or char == "'":
                if not in_string:
                    in_string = True
                    string_char = char
                elif char == string_char:
                    in_string = False
                    string_char = None
                continue
            
            if in_string:
                continue
            
            if char == '{':
                brace_count += 1
            elif char == '}':
                brace_count -= 1
                if brace_count == 0:
                    return i + 1
        
        return len(content)
    
    def _analyze_function(self, func_content: str, func_name: str, file_path: str) -> List[Vulnerability]:
        """Analyze a function for side-channel vulnerabilities"""
        vulnerabilities = []
        
        # Look for secret-dependent control flow
        if self._has_secret_dependent_control_flow(func_content):
            vuln = Vulnerability(
                file_path=file_path,
                line_number=0,  # Will be refined later
                vulnerability_type=VulnerabilityType.SECRET_DEPENDENT_BRANCH,
                severity=Severity.MEDIUM,
                description=f"Function '{func_name}' contains secret-dependent control flow",
                code_snippet=func_content[:200] + "..." if len(func_content) > 200 else func_content,
                context=f"Function definition: {func_name}",
                recommendation="Refactor to use constant-time operations and avoid secret-dependent branches",
                confidence=0.7
            )
            vulnerabilities.append(vuln)
        
        return vulnerabilities
    
    def _has_secret_dependent_control_flow(self, content: str) -> bool:
        """Check if content has secret-dependent control flow"""
        # Look for conditional statements with secret data
        conditional_patterns = [
            r'\bif\s*\([^)]*[a-zA-Z_]\w*[^)]*\)',
            r'\bwhile\s*\([^)]*[a-zA-Z_]\w*[^)]*\)',
            r'\bfor\s*\([^)]*[a-zA-Z_]\w*[^)]*\)',
        ]
        
        for pattern in conditional_patterns:
            if re.search(pattern, content, re.IGNORECASE):
                return True
        
        return False
    
    def _create_vulnerability(
        self, 
        file_path: str, 
        line_num: int, 
        vuln_type: VulnerabilityType, 
        line: str, 
        pattern: re.Pattern
    ) -> Optional[Vulnerability]:
        """Create a vulnerability object from detected pattern"""
        # Determine severity based on vulnerability type and context
        severity = self._determine_severity(vuln_type, line)
        
        # Determine confidence based on pattern match quality
        confidence = self._determine_confidence(pattern, line)
        
        # Generate description and recommendation
        description, recommendation = self._generate_description_and_recommendation(vuln_type, line)
        
        vuln = Vulnerability(
            file_path=file_path,
            line_number=line_num,
            vulnerability_type=vuln_type,
            severity=severity,
            description=description,
            code_snippet=line.strip(),
            context=f"Line {line_num} in {file_path}",
            recommendation=recommendation,
            confidence=confidence
        )
        
        return vuln
    
    def _determine_severity(self, vuln_type: VulnerabilityType, line: str) -> Severity:
        """Determine the severity of a detected vulnerability"""
        # High severity for critical patterns
        if vuln_type in [VulnerabilityType.SECRET_DEPENDENT_BRANCH, 
                        VulnerabilityType.SECRET_DEPENDENT_TABLE_LOOKUP]:
            return Severity.HIGH
        
        # Medium severity for other patterns
        if vuln_type in [VulnerabilityType.SECRET_DEPENDENT_MEMORY_ACCESS,
                        VulnerabilityType.SECRET_DEPENDENT_LOOP]:
            return Severity.MEDIUM
        
        # Low severity for less critical patterns
        return Severity.LOW
    
    def _determine_confidence(self, pattern: re.Pattern, line: str) -> float:
        """Determine confidence level of the detection"""
        # Base confidence on pattern match quality
        match = pattern.search(line)
        if not match:
            return 0.0
        
        # Higher confidence for longer, more specific matches
        match_length = len(match.group(0))
        line_length = len(line)
        
        if match_length / line_length > 0.5:
            return 0.9
        elif match_length / line_length > 0.3:
            return 0.7
        else:
            return 0.5
    
    def _generate_description_and_recommendation(
        self, vuln_type: VulnerabilityType, line: str
    ) -> Tuple[str, str]:
        """Generate description and recommendation for a vulnerability"""
        descriptions = {
            VulnerabilityType.SECRET_DEPENDENT_BRANCH: 
                "Secret-dependent branch detected that could leak timing information",
            VulnerabilityType.SECRET_DEPENDENT_TABLE_LOOKUP: 
                "Secret-dependent table lookup that could leak cache information",
            VulnerabilityType.SECRET_DEPENDENT_MEMORY_ACCESS: 
                "Secret-dependent memory access that could leak access patterns",
            VulnerabilityType.SECRET_DEPENDENT_LOOP: 
                "Secret-dependent loop that could leak iteration count",
            VulnerabilityType.SECRET_DEPENDENT_FUNCTION_CALL: 
                "Secret-dependent function call that could leak execution path",
            VulnerabilityType.TIMING_LEAK: 
                "Potential timing leak through early return or conditional execution",
            VulnerabilityType.CACHE_LEAK: 
                "Potential cache leak through memory access patterns",
        }
        
        recommendations = {
            VulnerabilityType.SECRET_DEPENDENT_BRANCH: 
                "Use constant-time operations and avoid secret-dependent control flow",
            VulnerabilityType.SECRET_DEPENDENT_TABLE_LOOKUP: 
                "Use constant-time table lookups or avoid secret-dependent indexing",
            VulnerabilityType.SECRET_DEPENDENT_MEMORY_ACCESS: 
                "Use constant-time memory access patterns",
            VulnerabilityType.SECRET_DEPENDENT_LOOP: 
                "Use constant-time loops or avoid secret-dependent bounds",
            VulnerabilityType.SECRET_DEPENDENT_FUNCTION_CALL: 
                "Use constant-time function calls or avoid secret-dependent parameters",
            VulnerabilityType.TIMING_LEAK: 
                "Ensure constant-time execution regardless of secret values",
            VulnerabilityType.CACHE_LEAK: 
                "Use cache-oblivious algorithms or constant-time memory access",
        }
        
        return descriptions.get(vuln_type, "Unknown vulnerability"), \
               recommendations.get(vuln_type, "Review and fix the identified issue")
    
    def scan_directory(self, directory: str, file_extensions: List[str] = None) -> ScanResult:
        """Scan a directory for side-channel vulnerabilities"""
        if file_extensions is None:
            file_extensions = ['.rs', '.asm', '.ll', '.s', '.o']
        
        start_time = time.time()
        vulnerabilities = []
        files_scanned = 0
        
        directory_path = Path(directory)
        if not directory_path.exists():
            logger.error(f"Directory does not exist: {directory}")
            return ScanResult(
                total_files_scanned=0,
                total_vulnerabilities=0,
                vulnerabilities_by_type={},
                vulnerabilities_by_severity={},
                vulnerabilities=[],
                scan_duration=0.0,
                timestamp=time.strftime('%Y-%m-%d %H:%M:%S')
            )
        
        # Find all relevant files
        for file_path in directory_path.rglob('*'):
            if file_path.is_file() and file_path.suffix in file_extensions:
                logger.info(f"Scanning file: {file_path}")
                file_vulns = self.scan_file(str(file_path))
                vulnerabilities.extend(file_vulns)
                files_scanned += 1
        
        # Calculate statistics
        vulnerabilities_by_type = {}
        vulnerabilities_by_severity = {}
        
        for vuln in vulnerabilities:
            vuln_type = vuln.vulnerability_type.value
            vuln_severity = vuln.severity.value
            
            vulnerabilities_by_type[vuln_type] = vulnerabilities_by_type.get(vuln_type, 0) + 1
            vulnerabilities_by_severity[vuln_severity] = vulnerabilities_by_severity.get(vuln_severity, 0) + 1
        
        scan_duration = time.time() - start_time
        
        result = ScanResult(
            total_files_scanned=files_scanned,
            total_vulnerabilities=len(vulnerabilities),
            vulnerabilities_by_type=vulnerabilities_by_type,
            vulnerabilities_by_severity=vulnerabilities_by_severity,
            vulnerabilities=vulnerabilities,
            scan_duration=scan_duration,
            timestamp=time.strftime('%Y-%m-%d %H:%M:%S')
        )
        
        return result

def main():
    """Main entry point for the side-channel scanner"""
    parser = argparse.ArgumentParser(
        description='Scan for side-channel vulnerabilities in Rust/LLVM code'
    )
    parser.add_argument('--asm-dir', help='Directory containing assembly files')
    parser.add_argument('--llvm-dir', help='Directory containing LLVM-IR files')
    parser.add_argument('--output', help='Output file for results (JSON)')
    parser.add_argument('--verbose', action='store_true', help='Enable verbose logging')
    parser.add_argument('--check-waivers', action='store_true', 
                       help='Check for existing waivers in /docs/waivers/SIDECHAN.md')
    
    args = parser.parse_args()
    
    if not args.asm_dir and not args.llvm_dir:
        parser.error("Must specify at least one directory to scan")
    
    # Initialize scanner
    scanner = SideChannelScanner(verbose=args.verbose)
    
    # Scan directories
    all_vulnerabilities = []
    
    if args.asm_dir:
        logger.info(f"Scanning assembly directory: {args.asm_dir}")
        asm_result = scanner.scan_directory(args.asm_dir, ['.asm', '.s', '.o'])
        all_vulnerabilities.extend(asm_result.vulnerabilities)
    
    if args.llvm_dir:
        logger.info(f"Scanning LLVM-IR directory: {args.llvm_dir}")
        llvm_result = scanner.scan_directory(args.llvm_dir, ['.ll', '.bc'])
        all_vulnerabilities.extend(llvm_result.vulnerabilities)
    
    # Check for waivers if requested
    if args.check_waivers:
        waived_vulnerabilities = check_waivers(all_vulnerabilities)
        all_vulnerabilities = [v for v in all_vulnerabilities if v not in waived_vulnerabilities]
    
    # Generate results
    result = ScanResult(
        total_files_scanned=(args.asm_dir and 1 or 0) + (args.llvm_dir and 1 or 0),
        total_vulnerabilities=len(all_vulnerabilities),
        vulnerabilities_by_type={},
        vulnerabilities_by_severity={},
        vulnerabilities=all_vulnerabilities,
        scan_duration=0.0,
        timestamp=time.strftime('%Y-%m-%d %H:%M:%S')
    )
    
    # Calculate statistics
    for vuln in all_vulnerabilities:
        vuln_type = vuln.vulnerability_type.value
        vuln_severity = vuln.severity.value
        
        result.vulnerabilities_by_type[vuln_type] = result.vulnerabilities_by_type.get(vuln_type, 0) + 1
        result.vulnerabilities_by_severity[vuln_severity] = result.vulnerabilities_by_severity.get(vuln_severity, 0) + 1
    
    # Output results
    if args.output:
        with open(args.output, 'w') as f:
            json.dump(asdict(result), f, indent=2, default=str)
        logger.info(f"Results written to: {args.output}")
    else:
        print(json.dumps(asdict(result), indent=2, default=str))
    
    # Exit with appropriate code
    if all_vulnerabilities:
        logger.warning(f"Found {len(all_vulnerabilities)} potential side-channel vulnerabilities")
        sys.exit(1)
    else:
        logger.info("No side-channel vulnerabilities detected")
        sys.exit(0)

def check_waivers(vulnerabilities: List[Vulnerability]) -> List[Vulnerability]:
    """Check for existing waivers in the documentation"""
    waiver_file = Path('/docs/waivers/SIDECHAN.md')
    waived_vulnerabilities = []
    
    if not waiver_file.exists():
        logger.warning("Waiver file not found: /docs/waivers/SIDECHAN.md")
        return waived_vulnerabilities
    
    try:
        with open(waiver_file, 'r') as f:
            waiver_content = f.read()
        
        # Simple waiver checking - look for file paths and line numbers
        for vuln in vulnerabilities:
            file_name = Path(vuln.file_path).name
            if file_name in waiver_content and str(vuln.line_number) in waiver_content:
                waived_vulnerabilities.append(vuln)
                logger.info(f"Vulnerability in {vuln.file_path}:{vuln.line_number} is waived")
    
    except Exception as e:
        logger.error(f"Error reading waiver file: {e}")
    
    return waived_vulnerabilities

if __name__ == '__main__':
    import time
    main()

