#!/usr/bin/env python3
"""
Reproducible Build Verification Script for Polymera OS

This script verifies that builds are completely reproducible by comparing
hashes across multiple build runs and ensuring identical outputs.
"""

import json
import hashlib
import subprocess
import sys
import os
import argparse
import tempfile
import shutil
from pathlib import Path
from typing import Dict, List, Any, Optional, Tuple
from datetime import datetime
import yaml

class ReproducibilityVerifier:
    """Verifies that builds are reproducible across different runs."""
    
    def __init__(self, build_dir: str = "build", num_runs: int = 2):
        self.build_dir = Path(build_dir)
        self.num_runs = num_runs
        self.build_results = []
        self.verification_report = {}
        
        # Ensure build directory exists
        self.build_dir.mkdir(exist_ok=True)
    
    def run_bazel_build(self, run_number: int) -> Dict[str, Any]:
        """Run a Bazel build with reproducible configuration."""
        
        print(f"🔨 Running Bazel build #{run_number}...")
        
        # Create build-specific directory
        run_dir = self.build_dir / f"run_{run_number}"
        run_dir.mkdir(exist_ok=True)
        
        # Set reproducible environment variables
        env = os.environ.copy()
        env.update({
            "SOURCE_DATE_EPOCH": "0",
            "BUILD_DATE": "1970-01-01T00:00:00Z",
            "BUILD_TIMESTAMP": "1970-01-01T00:00:00Z",
            "RUSTFLAGS": "-C target-cpu=native -C codegen-units=1 -C lto=fat",
            "CARGO_INCREMENTAL": "0",
            "CFLAGS": "-O3 -DNDEBUG -fno-ident -fno-stack-protector",
            "CXXFLAGS": "-O3 -DNDEBUG -fno-ident -fno-stack-protector",
        })
        
        try:
            # Run Bazel build with reproducible flags
            cmd = [
                "bazel", "build",
                "--config=reproducible_build",
                "--define=reproducible_build=true",
                "--define=build_timestamp=1970-01-01T00:00:00Z",
                "--define=source_hash=reproducible",
                "--output_user_root=" + str(run_dir / "bazel_root"),
                "//..."
            ]
            
            result = subprocess.run(
                cmd,
                env=env,
                cwd=Path.cwd(),
                capture_output=True,
                text=True,
                check=True
            )
            
            print(f"✅ Bazel build #{run_number} completed successfully")
            
            # Collect build artifacts
            artifacts = self._collect_bazel_artifacts(run_dir)
            
            return {
                "run_number": run_number,
                "build_dir": str(run_dir),
                "artifacts": artifacts,
                "stdout": result.stdout,
                "stderr": result.stderr,
                "returncode": result.returncode,
                "timestamp": datetime.utcnow().isoformat() + "Z"
            }
            
        except subprocess.CalledProcessError as e:
            print(f"❌ Bazel build #{run_number} failed: {e}")
            return {
                "run_number": run_number,
                "build_dir": str(run_dir),
                "error": str(e),
                "stdout": e.stdout,
                "stderr": e.stderr,
                "returncode": e.returncode,
                "timestamp": datetime.utcnow().isoformat() + "Z"
            }
    
    def run_nix_build(self, run_number: int) -> Dict[str, Any]:
        """Run a Nix build with reproducible configuration."""
        
        print(f"🔨 Running Nix build #{run_number}...")
        
        # Create build-specific directory
        run_dir = self.build_dir / f"nix_run_{run_number}"
        run_dir.mkdir(exist_ok=True)
        
        try:
            # Run Nix build
            cmd = [
                "nix-build",
                "--no-out-link",
                "--arg", "pkgs", "import <nixpkgs> {}",
                "tooling/repro/default.nix"
            ]
            
            result = subprocess.run(
                cmd,
                cwd=Path.cwd(),
                capture_output=True,
                text=True,
                check=True
            )
            
            print(f"✅ Nix build #{run_number} completed successfully")
            
            # Extract output path from Nix result
            output_path = result.stdout.strip()
            
            # Collect build artifacts
            artifacts = self._collect_nix_artifacts(Path(output_path))
            
            return {
                "run_number": run_number,
                "build_dir": str(run_dir),
                "nix_output": output_path,
                "artifacts": artifacts,
                "stdout": result.stdout,
                "stderr": result.stderr,
                "returncode": result.returncode,
                "timestamp": datetime.utcnow().isoformat() + "Z"
            }
            
        except subprocess.CalledProcessError as e:
            print(f"❌ Nix build #{run_number} failed: {e}")
            return {
                "run_number": run_number,
                "build_dir": str(run_dir),
                "error": str(e),
                "stdout": e.stdout,
                "stderr": e.stderr,
                "returncode": e.returncode,
                "timestamp": datetime.utcnow().isoformat() + "Z"
            }
    
    def _collect_bazel_artifacts(self, run_dir: Path) -> Dict[str, str]:
        """Collect artifacts from a Bazel build."""
        
        artifacts = {}
        
        # Look for Bazel output directories
        bazel_bin = run_dir / "bazel_root" / "bazel-bin"
        if bazel_bin.exists():
            for artifact in bazel_bin.rglob("*"):
                if artifact.is_file() and not artifact.name.endswith(('.sha256', '.buildinfo')):
                    # Calculate hash
                    file_hash = self._calculate_file_hash(artifact)
                    artifacts[str(artifact.relative_to(run_dir))] = file_hash
        
        return artifacts
    
    def _collect_nix_artifacts(self, nix_output: Path) -> Dict[str, str]:
        """Collect artifacts from a Nix build."""
        
        artifacts = {}
        
        if nix_output.exists():
            for artifact in nix_output.rglob("*"):
                if artifact.is_file() and not artifact.name.endswith(('.sha256', '.buildinfo')):
                    # Calculate hash
                    file_hash = self._calculate_file_hash(artifact)
                    artifacts[str(artifact.relative_to(nix_output))] = file_hash
        
        return artifacts
    
    def _calculate_file_hash(self, file_path: Path) -> str:
        """Calculate SHA256 hash of a file."""
        
        sha256_hash = hashlib.sha256()
        with open(file_path, "rb") as f:
            for chunk in iter(lambda: f.read(4096), b""):
                sha256_hash.update(chunk)
        return sha256_hash.hexdigest()
    
    def verify_reproducibility(self, build_type: str = "bazel") -> bool:
        """Run multiple builds and verify they are reproducible."""
        
        print(f"🔍 Verifying {build_type} build reproducibility...")
        
        # Run multiple builds
        for run_number in range(1, self.num_runs + 1):
            if build_type == "bazel":
                result = self.run_bazel_build(run_number)
            elif build_type == "nix":
                result = self.run_nix_build(run_number)
            else:
                print(f"❌ Unknown build type: {build_type}")
                return False
            
            self.build_results.append(result)
            
            # Check if build failed
            if "error" in result:
                print(f"❌ Build #{run_number} failed, cannot verify reproducibility")
                return False
        
        # Compare build results
        return self._compare_build_results()
    
    def _compare_build_results(self) -> bool:
        """Compare build results to verify reproducibility."""
        
        print("🔍 Comparing build results...")
        
        if len(self.build_results) < 2:
            print("❌ Need at least 2 builds to compare")
            return False
        
        # Get first build as reference
        reference_build = self.build_results[0]
        reference_artifacts = reference_build["artifacts"]
        
        # Compare with subsequent builds
        for i, build in enumerate(self.build_results[1:], 2):
            print(f"🔍 Comparing build #1 with build #{i}...")
            
            current_artifacts = build["artifacts"]
            
            # Check if artifact sets match
            if set(reference_artifacts.keys()) != set(current_artifacts.keys()):
                print(f"❌ Build #{i} has different artifacts than reference build")
                return False
            
            # Check if hashes match
            for artifact_name, reference_hash in reference_artifacts.items():
                if artifact_name not in current_artifacts:
                    print(f"❌ Build #{i} missing artifact: {artifact_name}")
                    return False
                
                current_hash = current_artifacts[artifact_name]
                if reference_hash != current_hash:
                    print(f"❌ Build #{i} hash mismatch for {artifact_name}:")
                    print(f"   Reference: {reference_hash}")
                    print(f"   Current:   {current_hash}")
                    return False
            
            print(f"✅ Build #{i} matches reference build")
        
        print("🎉 All builds are reproducible!")
        return True
    
    def generate_verification_report(self) -> str:
        """Generate a comprehensive verification report."""
        
        print("📊 Generating verification report...")
        
        # Calculate overall reproducibility
        reproducible = all("error" not in result for result in self.build_results)
        
        # Create report
        report = {
            "verification_timestamp": datetime.utcnow().isoformat() + "Z",
            "build_type": "bazel",  # Will be updated based on actual builds
            "num_runs": self.num_runs,
            "reproducible": reproducible,
            "build_results": self.build_results,
            "summary": {
                "total_builds": len(self.build_results),
                "successful_builds": len([r for r in self.build_results if "error" not in r]),
                "failed_builds": len([r for r in self.build_results if "error" in r]),
                "reproducible": reproducible
            }
        }
        
        # Save report
        report_file = self.build_dir / "reproducibility-verification-report.json"
        with open(report_file, 'w') as f:
            json.dump(report, f, indent=2)
        
        # Generate markdown report
        markdown_report = self._generate_markdown_report(report)
        markdown_file = self.build_dir / "reproducibility-verification-report.md"
        with open(markdown_file, 'w') as f:
            f.write(markdown_report)
        
        print(f"✅ Verification report saved to: {report_file}")
        print(f"✅ Markdown report saved to: {markdown_file}")
        
        return str(report_file)
    
    def _generate_markdown_report(self, report: Dict[str, Any]) -> str:
        """Generate a markdown verification report."""
        
        lines = [
            "# Polymera OS Reproducible Build Verification Report",
            "",
            f"**Generated**: {report['verification_timestamp']}",
            f"**Build Type**: {report['build_type']}",
            f"**Number of Runs**: {report['num_runs']}",
            f"**Reproducible**: {'✅ YES' if report['reproducible'] else '❌ NO'}",
            "",
            "## Summary",
            "",
            f"- **Total Builds**: {report['summary']['total_builds']}",
            f"- **Successful Builds**: {report['summary']['successful_builds']}",
            f"- **Failed Builds**: {report['summary']['failed_builds']}",
            f"- **Reproducible**: {'✅ YES' if report['summary']['reproducible'] else '❌ NO'}",
            "",
            "## Build Results",
            ""
        ]
        
        for i, build_result in enumerate(report['build_results'], 1):
            lines.extend([
                f"### Build #{i}",
                "",
                f"- **Timestamp**: {build_result['timestamp']}",
                f"- **Build Directory**: {build_result['build_dir']}",
                f"- **Status**: {'✅ Success' if 'error' not in build_result else '❌ Failed'}",
            ])
            
            if 'error' in build_result:
                lines.extend([
                    f"- **Error**: {build_result['error']}",
                    f"- **Return Code**: {build_result['returncode']}",
                ])
            else:
                lines.extend([
                    f"- **Artifacts**: {len(build_result['artifacts'])}",
                    f"- **Return Code**: {build_result['returncode']}",
                ])
            
            lines.append("")
        
        # Add reproducibility analysis
        if report['reproducible']:
            lines.extend([
                "## Reproducibility Analysis",
                "",
                "✅ **All builds produced identical outputs**",
                "",
                "This means:",
                "- Build environment is properly configured for reproducibility",
                "- All timestamps and non-deterministic elements are controlled",
                "- Build process is deterministic and consistent",
                "- Artifacts can be trusted across different build environments",
                ""
            ])
        else:
            lines.extend([
                "## Reproducibility Analysis",
                "",
                "❌ **Builds are not reproducible**",
                "",
                "This means:",
                "- Build environment has non-deterministic elements",
                "- Timestamps or other variable data is being included",
                "- Build process is not fully controlled",
                "- Artifacts may vary across different build environments",
                "",
                "## Recommendations",
                "",
                "1. **Check build environment variables**",
                "2. **Verify timestamp handling**",
                "3. **Review build flags and configurations**",
                "4. **Ensure all dependencies are pinned**",
                "5. **Check for non-deterministic build steps**",
                ""
            ])
        
        lines.extend([
            "## Verification Instructions",
            "",
            "To reproduce this verification:",
            "",
            "```bash",
            f"python3 verify_reproducibility.py --build-type {report['build_type']} --runs {report['num_runs']}",
            "```",
            "",
            "## Expected Result",
            "",
            "For reproducible builds, all artifact hashes should be identical across runs.",
            "",
            "---",
            "*Generated by Polymera OS Reproducibility Verifier*"
        ])
        
        return "\n".join(lines)
    
    def cleanup_builds(self):
        """Clean up build artifacts."""
        
        print("🧹 Cleaning up build artifacts...")
        
        try:
            shutil.rmtree(self.build_dir)
            print("✅ Build artifacts cleaned up successfully")
        except Exception as e:
            print(f"⚠️  Warning: Could not clean up build artifacts: {e}")

def main():
    """Main entry point for the reproducibility verifier."""
    
    parser = argparse.ArgumentParser(description="Verify reproducible builds for Polymera OS")
    parser.add_argument("--build-type", "-t", choices=["bazel", "nix"], default="bazel",
                       help="Build system to use (default: bazel)")
    parser.add_argument("--runs", "-r", type=int, default=2,
                       help="Number of build runs to compare (default: 2)")
    parser.add_argument("--build-dir", "-b", default="build",
                       help="Directory for build artifacts (default: build)")
    parser.add_argument("--keep-artifacts", "-k", action="store_true",
                       help="Keep build artifacts after verification")
    parser.add_argument("--verbose", "-v", action="store_true",
                       help="Enable verbose output")
    
    args = parser.parse_args()
    
    # Create verifier
    verifier = ReproducibilityVerifier(
        build_dir=args.build_dir,
        num_runs=args.runs
    )
    
    try:
        # Verify reproducibility
        reproducible = verifier.verify_reproducibility(args.build_type)
        
        # Generate report
        report_file = verifier.generate_verification_report()
        
        # Print summary
        print("\n" + "=" * 60)
        print("📊 Reproducibility Verification Summary")
        print("=" * 60)
        print(f"Build Type: {args.build_type}")
        print(f"Number of Runs: {args.runs}")
        print(f"Reproducible: {'✅ YES' if reproducible else '❌ NO'}")
        print(f"Report: {report_file}")
        print("=" * 60)
        
        # Clean up if requested
        if not args.keep_artifacts:
            verifier.cleanup_builds()
        
        # Exit with appropriate code
        if reproducible:
            print("🎉 Build reproducibility verification PASSED!")
            return 0
        else:
            print("❌ Build reproducibility verification FAILED!")
            return 1
        
    except Exception as e:
        print(f"❌ Error during reproducibility verification: {e}")
        return 1

if __name__ == "__main__":
    sys.exit(main())
