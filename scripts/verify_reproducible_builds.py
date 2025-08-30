#!/usr/bin/env python3
"""
Reproducible Build Verification Script

This script builds the kernel twice in clean environments and compares
artifact hashes and symbol maps to ensure reproducible builds.
"""

import sys
import os
import subprocess
import hashlib
import tempfile
import shutil
import json
import time
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass

@dataclass
class BuildArtifact:
    """Represents a build artifact with its hash and metadata"""
    path: str
    hash: str
    size: int
    mtime: float
    build_id: str

@dataclass
class SymbolMap:
    """Represents a symbol map with its content and hash"""
    path: str
    content: str
    hash: str
    symbol_count: int

class ReproducibleBuildVerifier:
    """Verifies that kernel builds are reproducible"""
    
    def __init__(self, kernel_dir: str = "kernel"):
        self.kernel_dir = Path(kernel_dir)
        self.build_dirs = []
        self.artifacts = []
        self.symbol_maps = []
        
        # Verify kernel directory exists
        if not self.kernel_dir.exists():
            raise FileNotFoundError(f"Kernel directory not found: {self.kernel_dir}")
        
        # Verify Cargo.toml exists
        cargo_toml = self.kernel_dir / "Cargo.toml"
        if not cargo_toml.exists():
            raise FileNotFoundError(f"Cargo.toml not found in {self.kernel_dir}")
    
    def create_clean_build_env(self, build_id: str) -> Path:
        """Create a clean build environment"""
        print(f"Creating clean build environment for build {build_id}...")
        
        # Create temporary build directory
        temp_dir = Path(tempfile.mkdtemp(prefix=f"polymera_build_{build_id}_"))
        self.build_dirs.append(temp_dir)
        
        # Copy kernel source to clean environment
        kernel_copy = temp_dir / "kernel"
        shutil.copytree(self.kernel_dir, kernel_copy)
        
        # Clean any existing build artifacts
        target_dir = kernel_copy / "target"
        if target_dir.exists():
            shutil.rmtree(target_dir)
        
        # Clean Cargo cache for this project
        cargo_cache = kernel_copy / ".cargo"
        if cargo_cache.exists():
            shutil.rmtree(cargo_cache)
        
        print(f"  Clean environment created at: {temp_dir}")
        return temp_dir
    
    def build_kernel(self, build_dir: Path, build_id: str) -> bool:
        """Build the kernel in the specified directory"""
        kernel_dir = build_dir / "kernel"
        
        print(f"Building kernel in {build_dir}...")
        
        try:
            # Set environment variables for reproducible builds
            env = os.environ.copy()
            env.update({
                'CARGO_TARGET_DIR': str(kernel_dir / "target"),
                'RUSTFLAGS': '-C target-cpu=native',
                'SOURCE_DATE_EPOCH': str(int(time.time())),
                'RUST_BACKTRACE': '1'
            })
            
            # Run cargo build
            cmd = [
                'cargo', 'build',
                '--target', 'x86_64-unknown-none',
                '--release',
                '--verbose'
            ]
            
            print(f"  Running: {' '.join(cmd)}")
            result = subprocess.run(
                cmd,
                cwd=kernel_dir,
                env=env,
                capture_output=True,
                text=True,
                timeout=300  # 5 minute timeout
            )
            
            if result.returncode != 0:
                print(f"  ❌ Build failed with return code {result.returncode}")
                print(f"  stdout: {result.stdout}")
                print(f"  stderr: {result.stderr}")
                return False
            
            print(f"  ✅ Build completed successfully")
            return True
            
        except subprocess.TimeoutExpired:
            print(f"  ❌ Build timed out after 5 minutes")
            return False
        except Exception as e:
            print(f"  ❌ Build failed with error: {e}")
            return False
    
    def collect_artifacts(self, build_dir: Path, build_id: str) -> List[BuildArtifact]:
        """Collect build artifacts and calculate hashes"""
        artifacts = []
        target_dir = build_dir / "kernel" / "target" / "x86_64-unknown-none" / "release"
        
        if not target_dir.exists():
            print(f"  Warning: Target directory not found: {target_dir}")
            return artifacts
        
        # Find all build artifacts
        artifact_patterns = [
            "*.elf", "*.bin", "*.o", "*.a", "*.so", "*.d", "*.map"
        ]
        
        for pattern in artifact_patterns:
            for artifact_path in target_dir.glob(pattern):
                if artifact_path.is_file():
                    artifact = self._analyze_artifact(artifact_path, build_id)
                    if artifact:
                        artifacts.append(artifact)
        
        # Also check for specific known artifacts
        known_artifacts = [
            "polymera-os-kernel",
            "polymera-os-kernel.d",
            "polymera-os-kernel.map"
        ]
        
        for artifact_name in known_artifacts:
            artifact_path = target_dir / artifact_name
            if artifact_path.exists() and artifact_path.is_file():
                artifact = self._analyze_artifact(artifact_path, build_id)
                if artifact and artifact not in artifacts:
                    artifacts.append(artifact)
        
        print(f"  Collected {len(artifacts)} artifacts")
        return artifacts
    
    def _analyze_artifact(self, artifact_path: Path, build_id: str) -> Optional[BuildArtifact]:
        """Analyze a single artifact and return BuildArtifact object"""
        try:
            # Calculate SHA256 hash
            sha256_hash = hashlib.sha256()
            with open(artifact_path, 'rb') as f:
                for chunk in iter(lambda: f.read(4096), b""):
                    sha256_hash.update(chunk)
            
            # Get file metadata
            stat = artifact_path.stat()
            
            artifact = BuildArtifact(
                path=str(artifact_path.relative_to(self.kernel_dir.parent)),
                hash=sha256_hash.hexdigest(),
                size=stat.st_size,
                mtime=stat.st_mtime,
                build_id=build_id
            )
            
            return artifact
            
        except Exception as e:
            print(f"    Warning: Could not analyze {artifact_path}: {e}")
            return None
    
    def collect_symbol_maps(self, build_dir: Path, build_id: str) -> List[SymbolMap]:
        """Collect symbol maps and their content"""
        symbol_maps = []
        target_dir = build_dir / "kernel" / "target" / "x86_64-unknown-none" / "release"
        
        # Look for symbol map files
        symbol_map_patterns = [
            "*.map", "*.sym", "*.nm", "*.objdump"
        ]
        
        for pattern in symbol_map_patterns:
            for map_path in target_dir.glob(pattern):
                if map_path.is_file():
                    symbol_map = self._analyze_symbol_map(map_path, build_id)
                    if symbol_map:
                        symbol_maps.append(symbol_map)
        
        # Also check for specific known symbol files
        known_symbol_files = [
            "polymera-os-kernel.map",
            "polymera-os-kernel.nm",
            "polymera-os-kernel.objdump"
        ]
        
        for symbol_name in known_symbol_files:
            symbol_path = target_dir / symbol_name
            if symbol_path.exists() and symbol_path.is_file():
                symbol_map = self._analyze_symbol_map(symbol_path, build_id)
                if symbol_map and symbol_map not in symbol_maps:
                    symbol_maps.append(symbol_map)
        
        print(f"  Collected {len(symbol_maps)} symbol maps")
        return symbol_maps
    
    def _analyze_symbol_map(self, map_path: Path, build_id: str) -> Optional[SymbolMap]:
        """Analyze a single symbol map file"""
        try:
            # Read file content
            with open(map_path, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
            
            # Calculate hash of content
            content_hash = hashlib.sha256(content.encode('utf-8')).hexdigest()
            
            # Count symbols (rough estimate)
            symbol_count = len([line for line in content.split('\n') 
                              if line.strip() and not line.startswith('#')])
            
            symbol_map = SymbolMap(
                path=str(map_path.relative_to(self.kernel_dir.parent)),
                content=content,
                hash=content_hash,
                symbol_count=symbol_count
            )
            
            return symbol_map
            
        except Exception as e:
            print(f"    Warning: Could not analyze symbol map {map_path}: {e}")
            return None
    
    def compare_artifacts(self, artifacts1: List[BuildArtifact], 
                         artifacts2: List[BuildArtifact]) -> Tuple[bool, List[str]]:
        """Compare artifacts between two builds"""
        print("\nComparing build artifacts...")
        
        # Group artifacts by path
        artifacts_by_path1 = {a.path: a for a in artifacts1}
        artifacts_by_path2 = {a.path: a for a in artifacts2}
        
        all_paths = set(artifacts_by_path1.keys()) | set(artifacts_by_path2.keys())
        differences = []
        
        for path in sorted(all_paths):
            if path not in artifacts_by_path1:
                differences.append(f"  ❌ {path}: Missing in build 1")
                continue
            
            if path not in artifacts_by_path2:
                differences.append(f"  ❌ {path}: Missing in build 2")
                continue
            
            artifact1 = artifacts_by_path1[path]
            artifact2 = artifacts_by_path2[path]
            
            if artifact1.hash != artifact2.hash:
                differences.append(f"  ❌ {path}: Hash mismatch")
                differences.append(f"    Build 1: {artifact1.hash}")
                differences.append(f"    Build 2: {artifact2.hash}")
                differences.append(f"    Size 1: {artifact1.size} bytes")
                differences.append(f"    Size 2: {artifact2.size} bytes")
            else:
                print(f"  ✅ {path}: Identical")
        
        is_identical = len(differences) == 0
        
        if is_identical:
            print("  ✅ All artifacts are identical between builds")
        else:
            print(f"  ❌ Found {len(differences)} differences in artifacts")
        
        return is_identical, differences
    
    def compare_symbol_maps(self, maps1: List[SymbolMap], 
                           maps2: List[SymbolMap]) -> Tuple[bool, List[str]]:
        """Compare symbol maps between two builds"""
        print("\nComparing symbol maps...")
        
        # Group symbol maps by path
        maps_by_path1 = {m.path: m for m in maps1}
        maps_by_path2 = {m.path: m for m in maps2}
        
        all_paths = set(maps_by_path1.keys()) | set(maps_by_path2.keys())
        differences = []
        
        for path in sorted(all_paths):
            if path not in maps_by_path1:
                differences.append(f"  ❌ {path}: Missing in build 1")
                continue
            
            if path not in maps_by_path2:
                differences.append(f"  ❌ {path}: Missing in build 2")
                continue
            
            map1 = maps_by_path1[path]
            map2 = maps_by_path2[path]
            
            if map1.hash != map2.hash:
                differences.append(f"  ❌ {path}: Content mismatch")
                differences.append(f"    Build 1: {map1.hash} ({map1.symbol_count} symbols)")
                differences.append(f"    Build 2: {map2.hash} ({map2.symbol_count} symbols)")
                
                # Show detailed differences
                content_diff = self._diff_symbol_maps(map1.content, map2.content)
                if content_diff:
                    differences.append("    Content differences:")
                    differences.extend(f"      {line}" for line in content_diff[:10])  # Limit output
                    if len(content_diff) > 10:
                        differences.append(f"      ... and {len(content_diff) - 10} more lines")
            else:
                print(f"  ✅ {path}: Identical ({map1.symbol_count} symbols)")
        
        is_identical = len(differences) == 0
        
        if is_identical:
            print("  ✅ All symbol maps are identical between builds")
        else:
            print(f"  ❌ Found {len(differences)} differences in symbol maps")
        
        return is_identical, differences
    
    def _diff_symbol_maps(self, content1: str, content2: str) -> List[str]:
        """Generate a simple diff between two symbol map contents"""
        lines1 = content1.split('\n')
        lines2 = content2.split('\n')
        
        differences = []
        
        # Simple line-by-line comparison
        max_lines = max(len(lines1), len(lines2))
        for i in range(max_lines):
            line1 = lines1[i] if i < len(lines1) else ""
            line2 = lines2[i] if i < len(lines2) else ""
            
            if line1 != line2:
                differences.append(f"Line {i+1}:")
                differences.append(f"  - {line1}")
                differences.append(f"  + {line2}")
        
        return differences
    
    def generate_report(self, artifacts1: List[BuildArtifact], 
                       artifacts2: List[BuildArtifact],
                       maps1: List[SymbolMap], 
                       maps2: List[SymbolMap],
                       artifacts_identical: bool,
                       maps_identical: bool,
                       artifact_differences: List[str],
                       map_differences: List[str]) -> str:
        """Generate a comprehensive verification report"""
        
        report = []
        report.append("# 🔍 Reproducible Build Verification Report")
        report.append("")
        report.append(f"**Generated**: {time.strftime('%Y-%m-%d %H:%M:%S UTC', time.gmtime())}")
        report.append(f"**Kernel Directory**: {self.kernel_dir}")
        report.append("")
        
        # Overall status
        overall_status = "✅ PASSED" if (artifacts_identical and maps_identical) else "❌ FAILED"
        report.append(f"## Overall Status: {overall_status}")
        report.append("")
        
        # Build summary
        report.append("## 📊 Build Summary")
        report.append("")
        report.append(f"- **Build 1 Artifacts**: {len(artifacts1)}")
        report.append(f"- **Build 2 Artifacts**: {len(artifacts2)}")
        report.append(f"- **Build 1 Symbol Maps**: {len(maps1)}")
        report.append(f"- **Build 2 Symbol Maps**: {len(maps2)}")
        report.append("")
        
        # Artifacts comparison
        report.append("## 🔧 Artifacts Comparison")
        report.append("")
        if artifacts_identical:
            report.append("✅ **All artifacts are identical between builds**")
        else:
            report.append("❌ **Artifact differences detected**")
            report.append("")
            report.append("### Differences:")
            for diff in artifact_differences:
                report.append(diff)
        report.append("")
        
        # Symbol maps comparison
        report.append("## 🗺️ Symbol Maps Comparison")
        report.append("")
        if maps_identical:
            report.append("✅ **All symbol maps are identical between builds**")
        else:
            report.append("❌ **Symbol map differences detected**")
            report.append("")
            report.append("### Differences:")
            for diff in map_differences:
                report.append(diff)
        report.append("")
        
        # Detailed artifact list
        report.append("## 📋 Artifact Details")
        report.append("")
        report.append("### Build 1 Artifacts:")
        for artifact in sorted(artifacts1, key=lambda x: x.path):
            report.append(f"- **{artifact.path}**")
            report.append(f"  - Hash: `{artifact.hash}`")
            report.append(f"  - Size: {artifact.size} bytes")
            report.append(f"  - Modified: {time.strftime('%Y-%m-%d %H:%M:%S', time.gmtime(artifact.mtime))}")
        report.append("")
        
        report.append("### Build 2 Artifacts:")
        for artifact in sorted(artifacts2, key=lambda x: x.path):
            report.append(f"- **{artifact.path}**")
            report.append(f"  - Hash: `{artifact.hash}`")
            report.append(f"  - Size: {artifact.size} bytes")
            report.append(f"  - Modified: {time.strftime('%Y-%m-%d %H:%M:%S', time.gmtime(artifact.mtime))}")
        report.append("")
        
        # Recommendations
        report.append("## 💡 Recommendations")
        report.append("")
        if artifacts_identical and maps_identical:
            report.append("✅ **Build is reproducible**")
            report.append("- Continue with confidence")
            report.append("- Consider adding this verification to CI/CD")
            report.append("- Document build environment requirements")
        else:
            report.append("❌ **Build is not reproducible**")
            report.append("- Investigate source of non-determinism")
            report.append("- Check for timestamp dependencies")
            report.append("- Verify build tool versions")
            report.append("- Review build scripts for non-deterministic operations")
        report.append("")
        
        # Footer
        report.append("---")
        report.append("*This report was generated by the Reproducible Build Verification system*")
        report.append("*For questions or issues, contact the Polymera OS development team*")
        
        return "\n".join(report)
    
    def cleanup(self):
        """Clean up temporary build directories"""
        print("\nCleaning up temporary build directories...")
        
        for build_dir in self.build_dirs:
            try:
                if build_dir.exists():
                    shutil.rmtree(build_dir)
                    print(f"  Cleaned: {build_dir}")
            except Exception as e:
                print(f"  Warning: Could not clean {build_dir}: {e}")
    
    def verify_reproducible_builds(self) -> bool:
        """Main verification method - builds kernel twice and compares results"""
        print("🔍 Starting reproducible build verification...")
        print(f"Kernel directory: {self.kernel_dir}")
        print("=" * 60)
        
        try:
            # Build 1
            print("\n📦 BUILD 1")
            print("-" * 30)
            build_dir1 = self.create_clean_build_env("1")
            
            if not self.build_kernel(build_dir1, "1"):
                print("❌ Build 1 failed - cannot continue verification")
                return False
            
            artifacts1 = self.collect_artifacts(build_dir1, "1")
            maps1 = self.collect_symbol_maps(build_dir1, "1")
            
            # Build 2
            print("\n📦 BUILD 2")
            print("-" * 30)
            build_dir2 = self.create_clean_build_env("2")
            
            if not self.build_kernel(build_dir2, "2"):
                print("❌ Build 2 failed - cannot continue verification")
                return False
            
            artifacts2 = self.collect_artifacts(build_dir2, "2")
            maps2 = self.collect_symbol_maps(build_dir2, "2")
            
            # Compare results
            print("\n🔍 COMPARISON")
            print("-" * 30)
            
            artifacts_identical, artifact_differences = self.compare_artifacts(artifacts1, artifacts2)
            maps_identical, map_differences = self.compare_symbol_maps(maps1, maps2)
            
            overall_success = artifacts_identical and maps_identical
            
            # Generate report
            report = self.generate_report(
                artifacts1, artifacts2, maps1, maps2,
                artifacts_identical, maps_identical,
                artifact_differences, map_differences
            )
            
            # Save report
            report_file = "reproducible_build_report.md"
            with open(report_file, 'w') as f:
                f.write(report)
            print(f"\n📄 Report saved to: {report_file}")
            
            # Print summary
            print("\n📊 VERIFICATION SUMMARY")
            print("=" * 60)
            if overall_success:
                print("✅ REPRODUCIBLE BUILD VERIFICATION PASSED")
                print("   All artifacts and symbol maps are identical between builds")
            else:
                print("❌ REPRODUCIBLE BUILD VERIFICATION FAILED")
                print("   Differences detected between builds")
                
                if not artifacts_identical:
                    print(f"   - {len(artifact_differences)} artifact differences")
                if not maps_identical:
                    print(f"   - {len(map_differences)} symbol map differences")
            
            return overall_success
            
        except Exception as e:
            print(f"❌ Verification failed with error: {e}")
            return False
        
        finally:
            self.cleanup()

def main():
    """Main entry point"""
    if len(sys.argv) > 2:
        print("Usage: python3 verify_reproducible_builds.py [kernel_directory]")
        print("Example: python3 verify_reproducible_builds.py kernel")
        sys.exit(1)
    
    kernel_dir = sys.argv[1] if len(sys.argv) == 2 else "kernel"
    
    try:
        # Create verifier
        verifier = ReproducibleBuildVerifier(kernel_dir)
        
        # Run verification
        success = verifier.verify_reproducible_builds()
        
        # Exit with appropriate code
        if success:
            print("\n🎉 Reproducible build verification completed successfully!")
            sys.exit(0)
        else:
            print("\n💥 Reproducible build verification failed!")
            print("   CI gate 'phase-1.5-repro' will fail")
            sys.exit(1)
            
    except Exception as e:
        print(f"❌ Fatal error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
