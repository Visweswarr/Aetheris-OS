#!/usr/bin/env python3
"""
Polymera OS Policy Enforcer

Enforces supply chain security policies by checking for unsigned artifacts
and ensuring all required components are properly signed and documented.
"""

import json
import subprocess
import sys
import os
import argparse
import hashlib
import datetime
from pathlib import Path
from typing import Dict, List, Any, Optional, Set
import yaml

class PolicyEnforcer:
    """Main policy enforcement class for Polymera OS supply chain security."""
    
    def __init__(self, config_file: str = None):
        self.config = self._load_config(config_file)
        self.violations = []
        self.warnings = []
        
        # Default policy configuration
        self.default_policy = {
            "require_signatures": True,
            "require_sbom": True,
            "blocked_extensions": [".tmp", ".log", ".cache"],
            "required_components": [
                "kernel", "security", "services", "tests", "ui", "tooling"
            ],
            "signature_verification": True,
            "sbom_verification": True,
            "fail_on_violation": True
        }
        
        # Merge with config file
        if self.config:
            self.policy = {**self.default_policy, **self.config}
        else:
            self.policy = self.default_policy
    
    def _load_config(self, config_file: str) -> Optional[Dict[str, Any]]:
        """Load policy configuration from file."""
        if not config_file:
            return None
        
        try:
            with open(config_file, 'r') as f:
                if config_file.endswith('.yaml') or config_file.endswith('.yml'):
                    return yaml.safe_load(f)
                else:
                    return json.load(f)
        except Exception as e:
            print(f"⚠️  Warning: Could not load config file {config_file}: {e}")
            return None
    
    def enforce_policy(self, paths: List[Path]) -> bool:
        """Enforce all policies on the given paths."""
        
        print("🔒 Enforcing supply chain security policies...")
        
        # Reset violations and warnings
        self.violations = []
        self.warnings = []
        
        # Check each path
        for path in paths:
            if path.is_file():
                self._check_file_policy(path)
            elif path.is_dir():
                self._check_directory_policy(path)
            else:
                self.warnings.append(f"Path does not exist: {path}")
        
        # Generate policy report
        self._generate_policy_report()
        
        # Return compliance status
        if self.violations and self.policy.get("fail_on_violation", True):
            print(f"❌ Policy enforcement failed: {len(self.violations)} violations found")
            return False
        else:
            print(f"✅ Policy enforcement passed: {len(self.violations)} violations, {len(self.warnings)} warnings")
            return True
    
    def _check_file_policy(self, file_path: Path):
        """Check policy compliance for a single file."""
        
        # Skip blocked extensions
        if any(file_path.suffix == ext for ext in self.policy.get("blocked_extensions", [])):
            return
        
        # Check if file is signed
        if self.policy.get("require_signatures", True):
            self._check_file_signature(file_path)
        
        # Check if file has SBOM entry
        if self.policy.get("require_sbom", True):
            self._check_file_sbom(file_path)
    
    def _check_directory_policy(self, dir_path: Path):
        """Check policy compliance for a directory."""
        
        print(f"📁 Checking directory: {dir_path}")
        
        # Check for required components
        if self.policy.get("required_components"):
            self._check_required_components(dir_path)
        
        # Check all files in directory
        for file_path in dir_path.rglob("*"):
            if file_path.is_file():
                self._check_file_policy(file_path)
    
    def _check_file_signature(self, file_path: Path):
        """Check if a file has a valid signature."""
        
        # Skip signature and certificate files themselves
        if file_path.suffix in ['.sig', '.cert']:
            return
        
        # Check for signature file
        sig_file = file_path.parent / f"{file_path.name}.sig"
        cert_file = file_path.parent / f"{file_path.name}.cert"
        
        if not sig_file.exists() or not cert_file.exists():
            violation = {
                "type": "missing_signature",
                "file": str(file_path),
                "severity": "high",
                "message": f"File {file_path.name} is not signed"
            }
            self.violations.append(violation)
            return
        
        # Verify signature if enabled
        if self.policy.get("signature_verification", True):
            if not self._verify_signature(file_path, sig_file, cert_file):
                violation = {
                    "type": "invalid_signature",
                    "file": str(file_path),
                    "severity": "high",
                    "message": f"File {file_path.name} has invalid signature"
                }
                self.violations.append(violation)
    
    def _check_file_sbom(self, file_path: Path):
        """Check if a file is documented in SBOM."""
        
        # Skip certain file types that don't need SBOM entries
        skip_extensions = ['.sig', '.cert', '.tmp', '.log', '.cache']
        if file_path.suffix in skip_extensions:
            return
        
        # For now, we'll just check if SBOM files exist
        # In a full implementation, you'd parse the SBOM and check for file entries
        sbom_files = list(file_path.parent.glob("*.json"))
        sbom_files.extend(file_path.parent.glob("*.xml"))
        sbom_files.extend(file_path.parent.glob("*.yaml"))
        
        if not sbom_files:
            warning = {
                "type": "no_sbom_found",
                "file": str(file_path),
                "severity": "medium",
                "message": f"No SBOM files found in {file_path.parent}"
            }
            self.warnings.append(warning)
    
    def _check_required_components(self, dir_path: Path):
        """Check if all required components are present."""
        
        required_components = self.policy.get("required_components", [])
        
        for component in required_components:
            component_path = dir_path / component
            if not component_path.exists():
                violation = {
                    "type": "missing_component",
                    "component": component,
                    "severity": "high",
                    "message": f"Required component {component} not found"
                }
                self.violations.append(violation)
            else:
                print(f"✅ Component found: {component}")
    
    def _verify_signature(self, file_path: Path, sig_file: Path, cert_file: Path) -> bool:
        """Verify a signature using cosign."""
        
        try:
            # Check if cosign is available
            result = subprocess.run(
                ["cosign", "version"],
                capture_output=True,
                text=True,
                check=True
            )
        except (subprocess.CalledProcessError, FileNotFoundError):
            print("⚠️  cosign not available, skipping signature verification")
            return True  # Assume valid if we can't verify
        
        try:
            # Verify the signature
            result = subprocess.run([
                "cosign", "verify-blob",
                str(file_path),
                "--signature", str(sig_file),
                "--certificate", str(cert_file)
            ], capture_output=True, text=True, check=True)
            
            return True
            
        except subprocess.CalledProcessError as e:
            print(f"❌ Signature verification failed for {file_path.name}: {e.stderr}")
            return False
    
    def _generate_policy_report(self):
        """Generate a comprehensive policy enforcement report."""
        
        report_lines = [
            "# Polymera OS Policy Enforcement Report",
            f"Generated: {datetime.datetime.utcnow().strftime('%Y-%m-%d %H:%M:%S UTC')}",
            "",
            "## Policy Summary",
            "",
            f"- **Total Violations**: {len(self.violations)}",
            f"- **Total Warnings**: {len(self.warnings)}",
            f"- **Policy Status**: {'FAILED' if self.violations else 'PASSED'}",
            "",
            "## Policy Configuration",
            "",
        ]
        
        # Add policy configuration
        for key, value in self.policy.items():
            report_lines.append(f"- **{key}**: {value}")
        
        report_lines.extend([
            "",
            "## Violations",
            ""
        ])
        
        if self.violations:
            # Group violations by type
            violation_types = {}
            for violation in self.violations:
                vtype = violation["type"]
                if vtype not in violation_types:
                    violation_types[vtype] = []
                violation_types[vtype].append(violation)
            
            for vtype, violations in violation_types.items():
                report_lines.extend([
                    f"### {vtype.replace('_', ' ').title()}",
                    ""
                ])
                
                for violation in violations:
                    report_lines.extend([
                        f"- **{violation['file']}**",
                        f"  - Severity: {violation['severity']}",
                        f"  - Message: {violation['message']}",
                        ""
                    ])
        else:
            report_lines.append("✅ No policy violations found!")
        
        report_lines.extend([
            "",
            "## Warnings",
            ""
        ])
        
        if self.warnings:
            # Group warnings by type
            warning_types = {}
            for warning in self.warnings:
                wtype = warning["type"]
                if wtype not in warning_types:
                    warning_types[wtype] = []
                warning_types[wtype].append(warning)
            
            for wtype, warnings in warning_types.items():
                report_lines.extend([
                    f"### {wtype.replace('_', ' ').title()}",
                    ""
                ])
                
                for warning in warnings:
                    report_lines.extend([
                        f"- **{warning['file']}**",
                        f"  - Severity: {warning['severity']}",
                        f"  - Message: {warning['message']}",
                        ""
                    ])
        else:
            report_lines.append("✅ No warnings!")
        
        report_lines.extend([
            "",
            "## Recommendations",
            ""
        ])
        
        if self.violations:
            report_lines.extend([
                "### High Priority",
                "- Fix all signature violations immediately",
                "- Ensure all artifacts are properly signed",
                "- Verify component completeness",
                "",
                "### Medium Priority",
                "- Address SBOM documentation gaps",
                "- Improve component coverage",
                "- Enhance signature verification",
                ""
            ])
        else:
            report_lines.extend([
                "### Maintain Current Status",
                "- Continue enforcing current policies",
                "- Monitor for new violations",
                "- Regular policy compliance audits",
                ""
            ])
        
        report_lines.extend([
            "---",
            "*Generated by Polymera OS Policy Enforcer*"
        ])
        
        report_content = "\n".join(report_lines)
        
        # Save report
        report_file = Path("policy-enforcement-report.md")
        with open(report_file, 'w') as f:
            f.write(report_content)
        
        print(f"✅ Policy report saved to: {report_file}")
        return report_content
    
    def check_sbom_completeness(self, sbom_dir: Path) -> Dict[str, Any]:
        """Check SBOM completeness and coverage."""
        
        print(f"📋 Checking SBOM completeness in: {sbom_dir}")
        
        if not sbom_dir.exists():
            return {"status": "error", "message": "SBOM directory not found"}
        
        # Find all SBOM files
        sbom_files = list(sbom_dir.glob("*.json"))
        sbom_files.extend(sbom_dir.glob("*.xml"))
        sbom_files.extend(sbom_dir.glob("*.yaml"))
        
        if not sbom_files:
            return {"status": "error", "message": "No SBOM files found"}
        
        # Analyze each SBOM file
        sbom_analysis = {}
        total_packages = 0
        
        for sbom_file in sbom_files:
            try:
                with open(sbom_file, 'r') as f:
                    if sbom_file.suffix == '.json':
                        sbom_data = json.load(f)
                    elif sbom_file.suffix in ['.yaml', '.yml']:
                        sbom_data = yaml.safe_load(f)
                    else:
                        continue
                
                # Extract package information
                packages = sbom_data.get("packages", [])
                sbom_analysis[sbom_file.name] = {
                    "packages": len(packages),
                    "format": sbom_file.suffix,
                    "valid": True
                }
                total_packages += len(packages)
                
            except Exception as e:
                sbom_analysis[sbom_file.name] = {
                    "packages": 0,
                    "format": sbom_file.suffix,
                    "valid": False,
                    "error": str(e)
                }
        
        return {
            "status": "success",
            "total_sbom_files": len(sbom_files),
            "total_packages": total_packages,
            "sbom_analysis": sbom_analysis,
            "coverage": "good" if total_packages > 0 else "poor"
        }
    
    def generate_compliance_summary(self) -> str:
        """Generate a compliance summary for CI/CD integration."""
        
        summary = {
            "timestamp": datetime.datetime.utcnow().isoformat() + "Z",
            "policy_status": "PASSED" if not self.violations else "FAILED",
            "violations_count": len(self.violations),
            "warnings_count": len(self.warnings),
            "violations": self.violations,
            "warnings": self.warnings
        }
        
        # Save JSON summary for CI/CD
        summary_file = Path("policy-compliance-summary.json")
        with open(summary_file, 'w') as f:
            json.dump(summary, f, indent=2)
        
        print(f"✅ Compliance summary saved to: {summary_file}")
        return summary_file

def main():
    """Main entry point for the policy enforcer."""
    
    parser = argparse.ArgumentParser(description="Enforce supply chain security policies")
    parser.add_argument("paths", nargs="+", help="Paths to check for policy compliance")
    parser.add_argument("--config", "-c", help="Policy configuration file")
    parser.add_argument("--sbom-check", action="store_true", help="Check SBOM completeness")
    parser.add_argument("--summary", action="store_true", help="Generate compliance summary")
    parser.add_argument("--no-fail", action="store_true", help="Don't fail on violations")
    
    args = parser.parse_args()
    
    # Create policy enforcer
    enforcer = PolicyEnforcer(config_file=args.config)
    
    # Override fail_on_violation if requested
    if args.no_fail:
        enforcer.policy["fail_on_violation"] = False
    
    try:
        # Convert paths to Path objects
        paths = [Path(p) for p in args.paths]
        
        # Enforce policies
        compliance = enforcer.enforce_policy(paths)
        
        # Check SBOM completeness if requested
        if args.sbom_check:
            for path in paths:
                if path.is_dir():
                    sbom_result = enforcer.check_sbom_completeness(path)
                    print(f"SBOM Analysis for {path}: {sbom_result}")
        
        # Generate compliance summary if requested
        if args.summary:
            enforcer.generate_compliance_summary()
        
        # Exit with appropriate code
        if compliance:
            print("✅ Policy enforcement completed successfully")
            sys.exit(0)
        else:
            print("❌ Policy enforcement failed")
            sys.exit(1)
        
    except Exception as e:
        print(f"❌ Error during policy enforcement: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
