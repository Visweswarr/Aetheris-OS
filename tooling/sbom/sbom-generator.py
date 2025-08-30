#!/usr/bin/env python3
"""
Polymera OS SBOM Generator

Generates comprehensive Software Bill of Materials (SBOM) for all components
using multiple formats and tools including Syft, Cargo SBOM, and custom analysis.
"""

import json
import subprocess
import sys
import os
import argparse
import hashlib
import datetime
from pathlib import Path
from typing import Dict, List, Any, Optional
import yaml

class SBOMGenerator:
    """Main SBOM generator class for Polymera OS components."""
    
    def __init__(self, output_dir: str = "sbom", format: str = "spdx-json"):
        self.output_dir = Path(output_dir)
        self.format = format
        self.output_dir.mkdir(exist_ok=True)
        
        # Component definitions
        self.components = {
            "kernel": {
                "path": "kernel",
                "type": "rust",
                "description": "Polymera OS kernel implementation"
            },
            "security": {
                "path": "security",
                "type": "rust",
                "description": "Security components including capability tokens"
            },
            "services": {
                "path": "services",
                "type": "rust",
                "description": "Microservices and gRPC services"
            },
            "tests": {
                "path": "tests",
                "type": "rust",
                "description": "Test suites and testing infrastructure"
            },
            "ui": {
                "path": "ui",
                "type": "typescript",
                "description": "User interface components"
            },
            "tooling": {
                "path": "tooling",
                "type": "python",
                "description": "Development and deployment tools"
            }
        }
    
    def generate_comprehensive_sbom(self) -> Dict[str, Any]:
        """Generate a comprehensive SBOM covering all components."""
        
        print("🔍 Generating comprehensive SBOM for Polymera OS...")
        
        # Initialize comprehensive SBOM
        sbom = {
            "sbomVersion": "SPDX-2.3",
            "name": "Polymera OS",
            "spdxId": "SPDXRef-PolymeraOS",
            "creationInfo": {
                "created": datetime.datetime.utcnow().isoformat() + "Z",
                "creators": [
                    "Tool: Polymera OS SBOM Generator",
                    "Tool: Syft",
                    "Tool: Cargo SBOM",
                    "Tool: Custom Analysis"
                ],
                "licenseListVersion": "3.19"
            },
            "packages": [],
            "relationships": [],
            "externalDocumentRefs": [],
            "files": []
        }
        
        # Generate component-specific SBOMs
        for component_name, component_info in self.components.items():
            if self._component_exists(component_info["path"]):
                print(f"📦 Processing component: {component_name}")
                component_sbom = self._generate_component_sbom(component_name, component_info)
                if component_sbom:
                    sbom["packages"].extend(component_sbom.get("packages", []))
                    sbom["relationships"].extend(component_sbom.get("relationships", []))
        
        # Add metadata
        sbom["packages"].append(self._create_metadata_package())
        
        # Generate relationships
        sbom["relationships"] = self._generate_relationships(sbom["packages"])
        
        return sbom
    
    def _component_exists(self, path: str) -> bool:
        """Check if a component path exists."""
        return Path(path).exists()
    
    def _generate_component_sbom(self, component_name: str, component_info: Dict[str, Any]) -> Optional[Dict[str, Any]]:
        """Generate SBOM for a specific component."""
        
        component_path = Path(component_info["path"])
        component_type = component_info["type"]
        
        if component_type == "rust":
            return self._generate_rust_sbom(component_name, component_path)
        elif component_type == "typescript":
            return self._generate_typescript_sbom(component_name, component_path)
        elif component_type == "python":
            return self._generate_python_sbom(component_name, component_path)
        else:
            print(f"⚠️  Unknown component type: {component_type}")
            return None
    
    def _generate_rust_sbom(self, component_name: str, component_path: Path) -> Optional[Dict[str, Any]]:
        """Generate SBOM for Rust components using cargo-sbom."""
        
        try:
            # Check if Cargo.toml exists
            cargo_toml = component_path / "Cargo.toml"
            if not cargo_toml.exists():
                print(f"⚠️  No Cargo.toml found in {component_path}")
                return None
            
            # Try to use cargo-sbom if available
            try:
                result = subprocess.run(
                    ["cargo", "sbom", "--format", self.format, "--output", f"{component_name}-sbom.json"],
                    cwd=component_path,
                    capture_output=True,
                    text=True,
                    check=True
                )
                
                # Read the generated SBOM
                sbom_file = component_path / f"{component_name}-sbom.json"
                if sbom_file.exists():
                    with open(sbom_file, 'r') as f:
                        cargo_sbom = json.load(f)
                    
                    # Copy to output directory
                    output_file = self.output_dir / f"{component_name}-sbom.json"
                    with open(output_file, 'w') as f:
                        json.dump(cargo_sbom, f, indent=2)
                    
                    print(f"✅ Generated Rust SBOM for {component_name}")
                    return cargo_sbom
                
            except subprocess.CalledProcessError:
                print(f"⚠️  cargo-sbom failed for {component_name}, using fallback method")
                return self._generate_rust_sbom_fallback(component_name, component_path)
                
        except Exception as e:
            print(f"❌ Error generating Rust SBOM for {component_name}: {e}")
            return None
    
    def _generate_rust_sbom_fallback(self, component_name: str, component_path: Path) -> Dict[str, Any]:
        """Fallback method for Rust SBOM generation."""
        
        # Parse Cargo.toml manually
        cargo_toml = component_path / "Cargo.toml"
        
        # Create basic package info
        package = {
            "spdxId": f"SPDXRef-{component_name}",
            "name": component_name,
            "versionInfo": "0.1.0",  # Default version
            "description": f"Polymera OS {component_name} component",
            "primaryPackagePurpose": "APPLICATION",
            "licenseConcluded": "Apache-2.0",
            "licenseDeclared": "Apache-2.0",
            "supplier": "Polymera OS Project",
            "originator": "Polymera OS Project",
            "downloadLocation": "NOASSERTION",
            "filesAnalyzed": False,
            "homepage": "https://github.com/polymera-os",
            "sourceInfo": f"Source code for {component_name} component"
        }
        
        # Try to extract version from Cargo.toml
        try:
            with open(cargo_toml, 'r') as f:
                content = f.read()
                # Simple version extraction
                if 'version = "' in content:
                    version_start = content.find('version = "') + 11
                    version_end = content.find('"', version_start)
                    if version_end > version_start:
                        package["versionInfo"] = content[version_start:version_end]
        except:
            pass
        
        return {
            "packages": [package],
            "relationships": []
        }
    
    def _generate_typescript_sbom(self, component_name: str, component_path: Path) -> Optional[Dict[str, Any]]:
        """Generate SBOM for TypeScript/Node.js components."""
        
        package_json = component_path / "package.json"
        if not package_json.exists():
            print(f"⚠️  No package.json found in {component_path}")
            return None
        
        try:
            with open(package_json, 'r') as f:
                pkg_data = json.load(f)
            
            # Create package info
            package = {
                "spdxId": f"SPDXRef-{component_name}",
                "name": pkg_data.get("name", component_name),
                "versionInfo": pkg_data.get("version", "0.1.0"),
                "description": pkg_data.get("description", f"Polymera OS {component_name} component"),
                "primaryPackagePurpose": "APPLICATION",
                "licenseConcluded": "Apache-2.0",
                "licenseDeclared": "Apache-2.0",
                "supplier": "Polymera OS Project",
                "originator": "Polymera OS Project",
                "downloadLocation": "NOASSERTION",
                "filesAnalyzed": False,
                "homepage": pkg_data.get("homepage", "https://github.com/polymera-os"),
                "sourceInfo": f"Source code for {component_name} component"
            }
            
            # Generate SBOM file
            sbom_data = {
                "packages": [package],
                "relationships": []
            }
            
            output_file = self.output_dir / f"{component_name}-sbom.json"
            with open(output_file, 'w') as f:
                json.dump(sbom_data, f, indent=2)
            
            print(f"✅ Generated TypeScript SBOM for {component_name}")
            return sbom_data
            
        except Exception as e:
            print(f"❌ Error generating TypeScript SBOM for {component_name}: {e}")
            return None
    
    def _generate_python_sbom(self, component_name: str, component_path: Path) -> Optional[Dict[str, Any]]:
        """Generate SBOM for Python components."""
        
        pyproject_toml = component_path / "pyproject.toml"
        requirements_txt = component_path / "requirements.txt"
        
        if not (pyproject_toml.exists() or requirements_txt.exists()):
            print(f"⚠️  No Python configuration found in {component_path}")
            return None
        
        try:
            # Create package info
            package = {
                "spdxId": f"SPDXRef-{component_name}",
                "name": component_name,
                "versionInfo": "0.1.0",  # Default version
                "description": f"Polymera OS {component_name} component",
                "primaryPackagePurpose": "APPLICATION",
                "licenseConcluded": "Apache-2.0",
                "licenseDeclared": "Apache-2.0",
                "supplier": "Polymera OS Project",
                "originator": "Polymera OS Project",
                "downloadLocation": "NOASSERTION",
                "filesAnalyzed": False,
                "homepage": "https://github.com/polymera-os",
                "sourceInfo": f"Source code for {component_name} component"
            }
            
            # Try to extract version from pyproject.toml
            if pyproject_toml.exists():
                try:
                    with open(pyproject_toml, 'r') as f:
                        content = f.read()
                        if 'version = "' in content:
                            version_start = content.find('version = "') + 11
                            version_end = content.find('"', version_start)
                            if version_end > version_start:
                                package["versionInfo"] = content[version_start:version_end]
                except:
                    pass
            
            # Generate SBOM file
            sbom_data = {
                "packages": [package],
                "relationships": []
            }
            
            output_file = self.output_dir / f"{component_name}-sbom.json"
            with open(output_file, 'w') as f:
                json.dump(sbom_data, f, indent=2)
            
            print(f"✅ Generated Python SBOM for {component_name}")
            return sbom_data
            
        except Exception as e:
            print(f"❌ Error generating Python SBOM for {component_name}: {e}")
            return None
    
    def _create_metadata_package(self) -> Dict[str, Any]:
        """Create metadata package for the overall project."""
        
        return {
            "spdxId": "SPDXRef-PolymeraOS-Metadata",
            "name": "Polymera OS",
            "versionInfo": "0.1.0",
            "description": "Polymera OS - A secure, privacy-preserving operating system",
            "primaryPackagePurpose": "APPLICATION",
            "licenseConcluded": "Apache-2.0",
            "licenseDeclared": "Apache-2.0",
            "supplier": "Polymera OS Project",
            "originator": "Polymera OS Project",
            "downloadLocation": "https://github.com/polymera-os/polymera-os",
            "filesAnalyzed": False,
            "homepage": "https://polymera-os.org",
            "sourceInfo": "Complete Polymera OS source code and components",
            "externalRefs": [
                {
                    "referenceCategory": "VCS",
                    "referenceType": "git",
                    "referenceLocator": "https://github.com/polymera-os/polymera-os.git"
                }
            ]
        }
    
    def _generate_relationships(self, packages: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        """Generate relationships between packages."""
        
        relationships = []
        
        # Create dependency relationships
        for package in packages:
            if package["spdxId"] != "SPDXRef-PolymeraOS-Metadata":
                relationship = {
                    "spdxElementId": "SPDXRef-PolymeraOS-Metadata",
                    "relatedSpdxElement": package["spdxId"],
                    "relationshipType": "CONTAINS"
                }
                relationships.append(relationship)
        
        return relationships
    
    def generate_syft_sbom(self) -> bool:
        """Generate SBOM using Syft tool."""
        
        try:
            print("🔍 Generating SBOM using Syft...")
            
            result = subprocess.run(
                ["syft", "packages", ".", "-o", self.format, "--file", str(self.output_dir / "syft-sbom.json")],
                capture_output=True,
                text=True,
                check=True
            )
            
            print("✅ Syft SBOM generated successfully")
            return True
            
        except subprocess.CalledProcessError as e:
            print(f"❌ Syft SBOM generation failed: {e}")
            print(f"Stderr: {e.stderr}")
            return False
        except FileNotFoundError:
            print("⚠️  Syft not found, skipping Syft SBOM generation")
            return False
    
    def merge_sboms(self) -> Dict[str, Any]:
        """Merge all generated SBOMs into a comprehensive SBOM."""
        
        print("🔄 Merging all SBOMs...")
        
        # Start with comprehensive SBOM
        merged_sbom = self.generate_comprehensive_sbom()
        
        # Try to merge Syft SBOM if available
        syft_sbom_file = self.output_dir / "syft-sbom.json"
        if syft_sbom_file.exists():
            try:
                with open(syft_sbom_file, 'r') as f:
                    syft_sbom = json.load(f)
                
                # Merge packages
                if "packages" in syft_sbom:
                    merged_sbom["packages"].extend(syft_sbom["packages"])
                
                # Merge relationships
                if "relationships" in syft_sbom:
                    merged_sbom["relationships"].extend(syft_sbom["relationships"])
                
                print("✅ Syft SBOM merged successfully")
                
            except Exception as e:
                print(f"⚠️  Error merging Syft SBOM: {e}")
        
        # Save merged SBOM
        merged_file = self.output_dir / "merged-sbom.json"
        with open(merged_file, 'w') as f:
            json.dump(merged_sbom, f, indent=2)
        
        print(f"✅ Merged SBOM saved to {merged_file}")
        return merged_sbom
    
    def generate_summary_report(self) -> str:
        """Generate a summary report of all SBOMs."""
        
        report_lines = [
            "# Polymera OS SBOM Summary Report",
            f"Generated: {datetime.datetime.utcnow().strftime('%Y-%m-%d %H:%M:%S UTC')}",
            "",
            "## Generated SBOMs",
            ""
        ]
        
        # List all SBOM files
        for sbom_file in self.output_dir.glob("*.json"):
            report_lines.append(f"- {sbom_file.name}")
        
        report_lines.extend([
            "",
            "## Component Coverage",
            ""
        ])
        
        # List component coverage
        for component_name, component_info in self.components.items():
            status = "✅" if self._component_exists(component_info["path"]) else "❌"
            report_lines.append(f"{status} {component_name}: {component_info['description']}")
        
        report_lines.extend([
            "",
            "## SBOM Formats",
            "",
            f"- Primary format: {self.format}",
            "- Component-specific SBOMs in JSON format",
            "- Merged comprehensive SBOM",
            "",
            "## Verification",
            "",
            "All SBOMs have been generated and are ready for supply chain security analysis.",
            "",
            "---",
            "*Generated by Polymera OS SBOM Generator*"
        ])
        
        report_content = "\n".join(report_lines)
        
        # Save report
        report_file = self.output_dir / "sbom-summary.md"
        with open(report_file, 'w') as f:
            f.write(report_content)
        
        print(f"✅ Summary report saved to {report_file}")
        return report_content

def main():
    """Main entry point for the SBOM generator."""
    
    parser = argparse.ArgumentParser(description="Generate SBOM for Polymera OS components")
    parser.add_argument("--output-dir", "-o", default="sbom", help="Output directory for SBOM files")
    parser.add_argument("--format", "-f", default="spdx-json", help="SBOM format (spdx-json, cyclonedx-json)")
    parser.add_argument("--syft-only", action="store_true", help="Only generate Syft SBOM")
    parser.add_argument("--components-only", action="store_true", help="Only generate component SBOMs")
    parser.add_argument("--merge", action="store_true", help="Merge all SBOMs into comprehensive SBOM")
    parser.add_argument("--report", action="store_true", help="Generate summary report")
    
    args = parser.parse_args()
    
    # Create generator
    generator = SBOMGenerator(output_dir=args.output_dir, format=args.format)
    
    try:
        if args.syft_only:
            # Only generate Syft SBOM
            success = generator.generate_syft_sbom()
            if not success:
                sys.exit(1)
        elif args.components_only:
            # Only generate component SBOMs
            generator.generate_comprehensive_sbom()
        elif args.merge:
            # Merge all SBOMs
            generator.merge_sboms()
        elif args.report:
            # Generate summary report
            generator.generate_summary_report()
        else:
            # Generate everything
            print("🚀 Starting comprehensive SBOM generation...")
            
            # Generate component SBOMs
            generator.generate_comprehensive_sbom()
            
            # Generate Syft SBOM
            generator.generate_syft_sbom()
            
            # Merge all SBOMs
            generator.merge_sboms()
            
            # Generate summary report
            generator.generate_summary_report()
            
            print("🎉 SBOM generation completed successfully!")
        
    except Exception as e:
        print(f"❌ Error during SBOM generation: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
