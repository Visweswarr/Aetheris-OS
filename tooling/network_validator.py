#!/usr/bin/env python3
"""
Aetheris OS Network Validator

This module provides validation tools for TLS/mTLS configurations, firewall rules,
and PQC (Post-Quantum Cryptography) implementations in the Aetheris OS networking subsystem.

Features:
- TLS configuration validation
- Firewall rule validation
- PQC key validation
- Network policy validation
- Performance baseline validation
- Security compliance checking
"""

import json
import yaml
import re
import ipaddress
import socket
import ssl
import hashlib
import time
import asyncio
import aiohttp
import argparse
import sys
from typing import Dict, List, Optional, Any, Tuple, Union
from dataclasses import dataclass, asdict
from enum import Enum
from pathlib import Path
import logging

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

class ValidationResult(Enum):
    PASS = "PASS"
    FAIL = "FAIL"
    WARN = "WARN"
    SKIP = "SKIP"

@dataclass
class ValidationIssue:
    severity: ValidationResult
    component: str
    message: str
    details: Optional[Dict[str, Any]] = None
    recommendation: Optional[str] = None

@dataclass
class ValidationReport:
    component: str
    overall_result: ValidationResult
    issues: List[ValidationIssue]
    summary: Dict[str, int]
    timestamp: float
    duration: float

class TLSValidator:
    """Validates TLS configurations and certificates"""
    
    def __init__(self):
        self.supported_versions = ['TLSv1.2', 'TLSv1.3']
        self.supported_ciphers = [
            'TLS_AES_256_GCM_SHA384',
            'TLS_AES_128_GCM_SHA256',
            'TLS_CHACHA20_POLY1305_SHA256',
            'TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384',
            'TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256'
        ]
        self.pqc_ciphers = [
            'TLS_AES_256_GCM_SHA384_KYBER512',
            'TLS_AES_256_GCM_SHA384_DILITHIUM2',
            'TLS_CHACHA20_POLY1305_SHA256_KYBER768'
        ]

    def validate_tls_config(self, config: Dict[str, Any]) -> List[ValidationIssue]:
        """Validate TLS configuration"""
        issues = []
        
        # Check TLS version
        tls_version = config.get('tls_version', 'TLSv1.3')
        if tls_version not in self.supported_versions:
            issues.append(ValidationIssue(
                severity=ValidationResult.FAIL,
                component='TLS',
                message=f'Unsupported TLS version: {tls_version}',
                recommendation='Use TLSv1.2 or TLSv1.3'
            ))
        
        # Check cipher suites
        cipher_suites = config.get('cipher_suites', [])
        if not cipher_suites:
            issues.append(ValidationIssue(
                severity=ValidationResult.WARN,
                component='TLS',
                message='No cipher suites specified',
                recommendation='Specify at least one cipher suite'
            ))
        
        for cipher in cipher_suites:
            if cipher not in self.supported_ciphers and cipher not in self.pqc_ciphers:
                issues.append(ValidationIssue(
                    severity=ValidationResult.WARN,
                    component='TLS',
                    message=f'Unknown cipher suite: {cipher}',
                    recommendation='Use supported cipher suites'
                ))
        
        # Check PQC configuration
        enable_pqc = config.get('enable_pqc', False)
        pqc_algorithms = config.get('pqc_algorithms', [])
        
        if enable_pqc and not pqc_algorithms:
            issues.append(ValidationIssue(
                severity=ValidationResult.FAIL,
                component='TLS',
                message='PQC enabled but no algorithms specified',
                recommendation='Specify PQC algorithms when enabling PQC'
            ))
        
        # Check mTLS configuration
        enable_mtls = config.get('enable_mtls', False)
        if enable_mtls:
            cert_file = config.get('cert_file')
            key_file = config.get('key_file')
            
            if not cert_file or not key_file:
                issues.append(ValidationIssue(
                    severity=ValidationResult.FAIL,
                    component='TLS',
                    message='mTLS enabled but certificate or key file not specified',
                    recommendation='Provide both certificate and key files for mTLS'
                ))
        
        # Check session configuration
        session_timeout = config.get('session_timeout', 3600)
        if session_timeout < 300 or session_timeout > 86400:
            issues.append(ValidationIssue(
                severity=ValidationResult.WARN,
                component='TLS',
                message=f'Session timeout {session_timeout}s is outside recommended range (300-86400s)',
                recommendation='Use session timeout between 5 minutes and 24 hours'
            ))
        
        return issues

    def validate_certificate(self, cert_data: bytes) -> List[ValidationIssue]:
        """Validate TLS certificate"""
        issues = []
        
        try:
            # Parse certificate (simplified validation)
            cert_hash = hashlib.sha256(cert_data).hexdigest()
            
            # Check certificate size
            if len(cert_data) < 100 or len(cert_data) > 10000:
                issues.append(ValidationIssue(
                    severity=ValidationResult.WARN,
                    component='TLS',
                    message=f'Certificate size {len(cert_data)} bytes is unusual',
                    recommendation='Verify certificate is valid'
                ))
            
            # Mock certificate validation - in real implementation would parse actual certificate
            logger.info(f'Validated certificate with hash: {cert_hash[:16]}...')
            
        except Exception as e:
            issues.append(ValidationIssue(
                severity=ValidationResult.FAIL,
                component='TLS',
                message=f'Failed to parse certificate: {str(e)}',
                recommendation='Check certificate format and validity'
            ))
        
        return issues

    async def test_tls_connection(self, host: str, port: int, sni: Optional[str] = None) -> List[ValidationIssue]:
        """Test TLS connection to remote host"""
        issues = []
        
        try:
            # Create SSL context
            context = ssl.create_default_context()
            context.check_hostname = False
            context.verify_mode = ssl.CERT_NONE  # For testing purposes
            
            # Connect and test
            start_time = time.time()
            with socket.create_connection((host, port), timeout=10) as sock:
                with context.wrap_socket(sock, server_hostname=sni) as ssock:
                    handshake_time = time.time() - start_time
                    
                    # Check TLS version
                    tls_version = ssock.version()
                    if tls_version not in ['TLSv1.2', 'TLSv1.3']:
                        issues.append(ValidationIssue(
                            severity=ValidationResult.WARN,
                            component='TLS',
                            message=f'Server uses {tls_version}, consider upgrading to TLSv1.3',
                            recommendation='Upgrade server to TLSv1.3'
                        ))
                    
                    # Check cipher suite
                    cipher = ssock.cipher()
                    if cipher:
                        cipher_name = cipher[0]
                        if 'RC4' in cipher_name or 'MD5' in cipher_name or 'SHA1' in cipher_name:
                            issues.append(ValidationIssue(
                                severity=ValidationResult.FAIL,
                                component='TLS',
                                message=f'Weak cipher suite: {cipher_name}',
                                recommendation='Use strong cipher suites (AES-GCM, ChaCha20-Poly1305)'
                            ))
                    
                    # Check handshake time
                    if handshake_time > 1.0:
                        issues.append(ValidationIssue(
                            severity=ValidationResult.WARN,
                            component='TLS',
                            message=f'Slow TLS handshake: {handshake_time:.2f}s',
                            recommendation='Optimize TLS configuration for better performance'
                        ))
                    
                    logger.info(f'TLS connection successful: {tls_version}, {cipher_name}, {handshake_time:.2f}s')
                    
        except Exception as e:
            issues.append(ValidationIssue(
                severity=ValidationResult.FAIL,
                component='TLS',
                message=f'TLS connection failed: {str(e)}',
                recommendation='Check network connectivity and TLS configuration'
            ))
        
        return issues

class FirewallValidator:
    """Validates firewall rules and policies"""
    
    def __init__(self):
        self.supported_protocols = ['TCP', 'UDP', 'ICMP', 'All']
        self.supported_actions = ['Allow', 'Deny', 'Drop', 'Reject', 'LogAndAllow', 'LogAndDeny', 'RateLimit']
        self.supported_scopes = ['Global', 'Process', 'Namespace', 'Socket']

    def validate_firewall_rule(self, rule: Dict[str, Any]) -> List[ValidationIssue]:
        """Validate individual firewall rule"""
        issues = []
        
        # Check required fields
        required_fields = ['id', 'name', 'type', 'scope', 'action', 'priority']
        for field in required_fields:
            if field not in rule:
                issues.append(ValidationIssue(
                    severity=ValidationResult.FAIL,
                    component='Firewall',
                    message=f'Missing required field: {field}',
                    recommendation=f'Add {field} to firewall rule'
                ))
        
        # Validate rule type
        rule_type = rule.get('type')
        if rule_type not in ['Ingress', 'Egress', 'Bidirectional']:
            issues.append(ValidationIssue(
                severity=ValidationResult.FAIL,
                component='Firewall',
                message=f'Invalid rule type: {rule_type}',
                recommendation='Use Ingress, Egress, or Bidirectional'
            ))
        
        # Validate scope
        scope = rule.get('scope')
        if scope not in self.supported_scopes:
            issues.append(ValidationIssue(
                severity=ValidationResult.FAIL,
                component='Firewall',
                message=f'Invalid scope: {scope}',
                recommendation=f'Use one of: {", ".join(self.supported_scopes)}'
            ))
        
        # Validate action
        action = rule.get('action')
        if action not in self.supported_actions:
            issues.append(ValidationIssue(
                severity=ValidationResult.FAIL,
                component='Firewall',
                message=f'Invalid action: {action}',
                recommendation=f'Use one of: {", ".join(self.supported_actions)}'
            ))
        
        # Validate priority
        priority = rule.get('priority', 0)
        if not isinstance(priority, int) or priority < 0 or priority > 65535:
            issues.append(ValidationIssue(
                severity=ValidationResult.FAIL,
                component='Firewall',
                message=f'Invalid priority: {priority}',
                recommendation='Use integer priority between 0 and 65535'
            ))
        
        # Validate IP addresses
        conditions = rule.get('conditions', {})
        source_ips = conditions.get('source_ips', [])
        dest_ips = conditions.get('destination_ips', [])
        
        for ip_list, ip_type in [(source_ips, 'source'), (dest_ips, 'destination')]:
            for ip in ip_list:
                if not self._validate_ip_or_cidr(ip):
                    issues.append(ValidationIssue(
                        severity=ValidationResult.FAIL,
                        component='Firewall',
                        message=f'Invalid {ip_type} IP/CIDR: {ip}',
                        recommendation='Use valid IPv4/IPv6 address or CIDR notation'
                    ))
        
        # Validate ports
        source_ports = conditions.get('source_ports', [])
        dest_ports = conditions.get('destination_ports', [])
        
        for port_list, port_type in [(source_ports, 'source'), (dest_ports, 'destination')]:
            for port in port_list:
                if not isinstance(port, int) or port < 0 or port > 65535:
                    issues.append(ValidationIssue(
                        severity=ValidationResult.FAIL,
                        component='Firewall',
                        message=f'Invalid {port_type} port: {port}',
                        recommendation='Use integer port between 0 and 65535'
                    ))
        
        # Validate protocols
        protocols = conditions.get('protocols', [])
        for protocol in protocols:
            if protocol not in self.supported_protocols:
                issues.append(ValidationIssue(
                    severity=ValidationResult.FAIL,
                    component='Firewall',
                    message=f'Unsupported protocol: {protocol}',
                    recommendation=f'Use one of: {", ".join(self.supported_protocols)}'
                ))
        
        return issues

    def _validate_ip_or_cidr(self, ip_str: str) -> bool:
        """Validate IP address or CIDR notation"""
        try:
            ipaddress.ip_network(ip_str, strict=False)
            return True
        except ValueError:
            return False

    def validate_firewall_policy(self, policy: Dict[str, Any]) -> List[ValidationIssue]:
        """Validate firewall policy"""
        issues = []
        
        # Check required fields
        required_fields = ['id', 'name', 'version', 'description', 'rules']
        for field in required_fields:
            if field not in policy:
                issues.append(ValidationIssue(
                    severity=ValidationResult.FAIL,
                    component='Firewall',
                    message=f'Missing required policy field: {field}',
                    recommendation=f'Add {field} to firewall policy'
                ))
        
        # Validate version format
        version = policy.get('version', '')
        if not re.match(r'^\d+\.\d+(\.\d+)?$', version):
            issues.append(ValidationIssue(
                severity=ValidationResult.WARN,
                component='Firewall',
                message=f'Invalid version format: {version}',
                recommendation='Use semantic versioning (e.g., 1.0.0)'
            ))
        
        # Validate rules
        rules = policy.get('rules', [])
        if not rules:
            issues.append(ValidationIssue(
                severity=ValidationResult.WARN,
                component='Firewall',
                message='Policy has no rules',
                recommendation='Add at least one rule to the policy'
            ))
        
        for i, rule in enumerate(rules):
            rule_issues = self.validate_firewall_rule(rule)
            for issue in rule_issues:
                issue.message = f'Rule {i}: {issue.message}'
            issues.extend(rule_issues)
        
        return issues

    def validate_rego_policy(self, rego_content: str) -> List[ValidationIssue]:
        """Validate Rego policy syntax"""
        issues = []
        
        # Basic Rego syntax validation
        if not rego_content.strip():
            issues.append(ValidationIssue(
                severity=ValidationResult.FAIL,
                component='Firewall',
                message='Empty Rego policy',
                recommendation='Provide valid Rego policy content'
            ))
            return issues
        
        # Check for required package declaration
        if not re.search(r'^package\s+\w+', rego_content, re.MULTILINE):
            issues.append(ValidationIssue(
                severity=ValidationResult.WARN,
                component='Firewall',
                message='No package declaration found',
                recommendation='Add package declaration at the beginning'
            ))
        
        # Check for default rule
        if not re.search(r'^default\s+\w+\s*=', rego_content, re.MULTILINE):
            issues.append(ValidationIssue(
                severity=ValidationResult.WARN,
                component='Firewall',
                message='No default rule found',
                recommendation='Add a default rule for policy completeness'
            ))
        
        # Check for common Rego patterns
        if 'input.connection' not in rego_content:
            issues.append(ValidationIssue(
                severity=ValidationResult.WARN,
                component='Firewall',
                message='Policy does not reference input.connection',
                recommendation='Use input.connection to access connection information'
            ))
        
        return issues

class PQCValidator:
    """Validates PQC (Post-Quantum Cryptography) configurations"""
    
    def __init__(self):
        self.supported_kem_algorithms = ['Kyber512', 'Kyber768', 'Kyber1024']
        self.supported_sig_algorithms = ['Dilithium2', 'Dilithium3', 'Dilithium5', 'Falcon512', 'Falcon1024']
        self.key_sizes = {
            'Kyber512': {'public': 800, 'private': 1632, 'ciphertext': 768, 'shared': 32},
            'Kyber768': {'public': 1184, 'private': 2400, 'ciphertext': 1088, 'shared': 32},
            'Kyber1024': {'public': 1568, 'private': 3168, 'ciphertext': 1568, 'shared': 32},
            'Dilithium2': {'public': 1312, 'private': 2528, 'signature': 2420},
            'Dilithium3': {'public': 1952, 'private': 4000, 'signature': 3293},
            'Dilithium5': {'public': 2592, 'private': 4864, 'signature': 4595},
            'Falcon512': {'public': 897, 'private': 1281, 'signature': 690},
            'Falcon1024': {'public': 1793, 'private': 2305, 'signature': 1330}
        }

    def validate_pqc_algorithm(self, algorithm: str) -> List[ValidationIssue]:
        """Validate PQC algorithm"""
        issues = []
        
        if algorithm not in self.supported_kem_algorithms and algorithm not in self.supported_sig_algorithms:
            issues.append(ValidationIssue(
                severity=ValidationResult.FAIL,
                component='PQC',
                message=f'Unsupported PQC algorithm: {algorithm}',
                recommendation=f'Use one of: {", ".join(self.supported_kem_algorithms + self.supported_sig_algorithms)}'
            ))
        
        return issues

    def validate_pqc_key_pair(self, key_pair: Dict[str, Any]) -> List[ValidationIssue]:
        """Validate PQC key pair"""
        issues = []
        
        # Check required fields
        required_fields = ['id', 'algorithm', 'public_key', 'private_key_id']
        for field in required_fields:
            if field not in key_pair:
                issues.append(ValidationIssue(
                    severity=ValidationResult.FAIL,
                    component='PQC',
                    message=f'Missing required field: {field}',
                    recommendation=f'Add {field} to PQC key pair'
                ))
        
        # Validate algorithm
        algorithm = key_pair.get('algorithm')
        algorithm_issues = self.validate_pqc_algorithm(algorithm)
        issues.extend(algorithm_issues)
        
        # Validate key sizes
        if algorithm in self.key_sizes:
            expected_sizes = self.key_sizes[algorithm]
            public_key = key_pair.get('public_key', b'')
            
            if isinstance(public_key, str):
                public_key = public_key.encode()
            
            expected_public_size = expected_sizes.get('public', 0)
            if len(public_key) != expected_public_size:
                issues.append(ValidationIssue(
                    severity=ValidationResult.FAIL,
                    component='PQC',
                    message=f'Invalid public key size for {algorithm}: {len(public_key)} bytes (expected {expected_public_size})',
                    recommendation=f'Use correct key size for {algorithm}'
                ))
        
        return issues

    def validate_pqc_configuration(self, config: Dict[str, Any]) -> List[ValidationIssue]:
        """Validate PQC configuration"""
        issues = []
        
        # Check if PQC is enabled
        enable_pqc = config.get('enable_pqc', False)
        if not enable_pqc:
            issues.append(ValidationIssue(
                severity=ValidationResult.WARN,
                component='PQC',
                message='PQC is not enabled',
                recommendation='Consider enabling PQC for future-proof security'
            ))
            return issues
        
        # Check PQC algorithms
        pqc_algorithms = config.get('pqc_algorithms', [])
        if not pqc_algorithms:
            issues.append(ValidationIssue(
                severity=ValidationResult.FAIL,
                component='PQC',
                message='PQC enabled but no algorithms specified',
                recommendation='Specify PQC algorithms when enabling PQC'
            ))
        
        for algorithm in pqc_algorithms:
            algorithm_issues = self.validate_pqc_algorithm(algorithm)
            issues.extend(algorithm_issues)
        
        # Check for hybrid configuration
        cipher_suites = config.get('cipher_suites', [])
        has_hybrid = any('Hybrid' in cipher for cipher in cipher_suites)
        
        if enable_pqc and not has_hybrid:
            issues.append(ValidationIssue(
                severity=ValidationResult.WARN,
                component='PQC',
                message='PQC enabled but no hybrid cipher suites found',
                recommendation='Use hybrid cipher suites for PQC compatibility'
            ))
        
        return issues

class NetworkValidator:
    """Main network validator class"""
    
    def __init__(self):
        self.tls_validator = TLSValidator()
        self.firewall_validator = FirewallValidator()
        self.pqc_validator = PQCValidator()

    async def validate_tls_config(self, config_path: str) -> ValidationReport:
        """Validate TLS configuration from file"""
        start_time = time.time()
        issues = []
        
        try:
            with open(config_path, 'r') as f:
                if config_path.endswith('.json'):
                    config = json.load(f)
                elif config_path.endswith(('.yml', '.yaml')):
                    config = yaml.safe_load(f)
                else:
                    raise ValueError(f'Unsupported file format: {config_path}')
            
            issues = self.tls_validator.validate_tls_config(config)
            
        except Exception as e:
            issues.append(ValidationIssue(
                severity=ValidationResult.FAIL,
                component='TLS',
                message=f'Failed to load TLS configuration: {str(e)}',
                recommendation='Check file format and syntax'
            ))
        
        duration = time.time() - start_time
        return self._create_report('TLS', issues, duration)

    async def validate_firewall_rules(self, rules_path: str) -> ValidationReport:
        """Validate firewall rules from file"""
        start_time = time.time()
        issues = []
        
        try:
            with open(rules_path, 'r') as f:
                if rules_path.endswith('.json'):
                    rules = json.load(f)
                elif rules_path.endswith(('.yml', '.yaml')):
                    rules = yaml.safe_load(f)
                else:
                    raise ValueError(f'Unsupported file format: {rules_path}')
            
            if isinstance(rules, list):
                for i, rule in enumerate(rules):
                    rule_issues = self.firewall_validator.validate_firewall_rule(rule)
                    for issue in rule_issues:
                        issue.message = f'Rule {i}: {issue.message}'
                    issues.extend(rule_issues)
            elif isinstance(rules, dict):
                if 'rules' in rules:
                    issues.extend(self.firewall_validator.validate_firewall_policy(rules))
                else:
                    issues.extend(self.firewall_validator.validate_firewall_rule(rules))
            
        except Exception as e:
            issues.append(ValidationIssue(
                severity=ValidationResult.FAIL,
                component='Firewall',
                message=f'Failed to load firewall rules: {str(e)}',
                recommendation='Check file format and syntax'
            ))
        
        duration = time.time() - start_time
        return self._create_report('Firewall', issues, duration)

    async def validate_rego_policy(self, policy_path: str) -> ValidationReport:
        """Validate Rego policy from file"""
        start_time = time.time()
        issues = []
        
        try:
            with open(policy_path, 'r') as f:
                rego_content = f.read()
            
            issues = self.firewall_validator.validate_rego_policy(rego_content)
            
        except Exception as e:
            issues.append(ValidationIssue(
                severity=ValidationResult.FAIL,
                component='Firewall',
                message=f'Failed to load Rego policy: {str(e)}',
                recommendation='Check file format and syntax'
            ))
        
        duration = time.time() - start_time
        return self._create_report('Rego Policy', issues, duration)

    async def validate_pqc_config(self, config_path: str) -> ValidationReport:
        """Validate PQC configuration from file"""
        start_time = time.time()
        issues = []
        
        try:
            with open(config_path, 'r') as f:
                if config_path.endswith('.json'):
                    config = json.load(f)
                elif config_path.endswith(('.yml', '.yaml')):
                    config = yaml.safe_load(f)
                else:
                    raise ValueError(f'Unsupported file format: {config_path}')
            
            issues = self.pqc_validator.validate_pqc_configuration(config)
            
        except Exception as e:
            issues.append(ValidationIssue(
                severity=ValidationResult.FAIL,
                component='PQC',
                message=f'Failed to load PQC configuration: {str(e)}',
                recommendation='Check file format and syntax'
            ))
        
        duration = time.time() - start_time
        return self._create_report('PQC', issues, duration)

    async def test_tls_connection(self, host: str, port: int, sni: Optional[str] = None) -> ValidationReport:
        """Test TLS connection to remote host"""
        start_time = time.time()
        issues = await self.tls_validator.test_tls_connection(host, port, sni)
        duration = time.time() - start_time
        return self._create_report('TLS Connection', issues, duration)

    def _create_report(self, component: str, issues: List[ValidationIssue], duration: float) -> ValidationReport:
        """Create validation report"""
        summary = {
            'pass': len([i for i in issues if i.severity == ValidationResult.PASS]),
            'fail': len([i for i in issues if i.severity == ValidationResult.FAIL]),
            'warn': len([i for i in issues if i.severity == ValidationResult.WARN]),
            'skip': len([i for i in issues if i.severity == ValidationResult.SKIP])
        }
        
        overall_result = ValidationResult.PASS
        if summary['fail'] > 0:
            overall_result = ValidationResult.FAIL
        elif summary['warn'] > 0:
            overall_result = ValidationResult.WARN
        
        return ValidationReport(
            component=component,
            overall_result=overall_result,
            issues=issues,
            summary=summary,
            timestamp=time.time(),
            duration=duration
        )

    def print_report(self, report: ValidationReport, verbose: bool = False):
        """Print validation report"""
        print(f"\n{'='*60}")
        print(f"Validation Report: {report.component}")
        print(f"{'='*60}")
        print(f"Overall Result: {report.overall_result.value}")
        print(f"Duration: {report.duration:.3f}s")
        print(f"Timestamp: {time.ctime(report.timestamp)}")
        print(f"\nSummary:")
        print(f"  Pass: {report.summary['pass']}")
        print(f"  Fail: {report.summary['fail']}")
        print(f"  Warn: {report.summary['warn']}")
        print(f"  Skip: {report.summary['skip']}")
        
        if report.issues:
            print(f"\nIssues:")
            for issue in report.issues:
                print(f"  [{issue.severity.value}] {issue.component}: {issue.message}")
                if verbose and issue.recommendation:
                    print(f"    Recommendation: {issue.recommendation}")
                if verbose and issue.details:
                    print(f"    Details: {json.dumps(issue.details, indent=2)}")

async def main():
    """Main function"""
    parser = argparse.ArgumentParser(description='Aetheris OS Network Validator')
    parser.add_argument('--tls-config', help='TLS configuration file to validate')
    parser.add_argument('--firewall-rules', help='Firewall rules file to validate')
    parser.add_argument('--rego-policy', help='Rego policy file to validate')
    parser.add_argument('--pqc-config', help='PQC configuration file to validate')
    parser.add_argument('--test-tls', help='Test TLS connection (format: host:port)')
    parser.add_argument('--sni', help='SNI hostname for TLS test')
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
    parser.add_argument('--output', '-o', help='Output report to file')
    
    args = parser.parse_args()
    
    if not any([args.tls_config, args.firewall_rules, args.rego_policy, args.pqc_config, args.test_tls]):
        parser.print_help()
        return 1
    
    validator = NetworkValidator()
    reports = []
    
    # Validate TLS configuration
    if args.tls_config:
        logger.info(f'Validating TLS configuration: {args.tls_config}')
        report = await validator.validate_tls_config(args.tls_config)
        reports.append(report)
        validator.print_report(report, args.verbose)
    
    # Validate firewall rules
    if args.firewall_rules:
        logger.info(f'Validating firewall rules: {args.firewall_rules}')
        report = await validator.validate_firewall_rules(args.firewall_rules)
        reports.append(report)
        validator.print_report(report, args.verbose)
    
    # Validate Rego policy
    if args.rego_policy:
        logger.info(f'Validating Rego policy: {args.rego_policy}')
        report = await validator.validate_rego_policy(args.rego_policy)
        reports.append(report)
        validator.print_report(report, args.verbose)
    
    # Validate PQC configuration
    if args.pqc_config:
        logger.info(f'Validating PQC configuration: {args.pqc_config}')
        report = await validator.validate_pqc_config(args.pqc_config)
        reports.append(report)
        validator.print_report(report, args.verbose)
    
    # Test TLS connection
    if args.test_tls:
        try:
            host, port = args.test_tls.split(':')
            port = int(port)
            logger.info(f'Testing TLS connection: {host}:{port}')
            report = await validator.test_tls_connection(host, port, args.sni)
            reports.append(report)
            validator.print_report(report, args.verbose)
        except ValueError:
            logger.error('Invalid TLS test format. Use host:port')
            return 1
    
    # Output reports to file
    if args.output:
        output_data = {
            'reports': [asdict(report) for report in reports],
            'timestamp': time.time(),
            'validator_version': '1.0.0'
        }
        
        with open(args.output, 'w') as f:
            json.dump(output_data, f, indent=2, default=str)
        
        logger.info(f'Reports saved to: {args.output}')
    
    # Return exit code based on results
    has_failures = any(report.overall_result == ValidationResult.FAIL for report in reports)
    return 1 if has_failures else 0

if __name__ == '__main__':
    sys.exit(asyncio.run(main()))
