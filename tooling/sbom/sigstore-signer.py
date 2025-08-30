#!/usr/bin/env python3
"""
Polymera OS Sigstore Signer

Signs all artifacts using Sigstore keyless signing for supply chain security.
Provides both individual and batch signing capabilities.
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

class SigstoreSigner:
    """Main Sigstore signing class for Polymera OS artifacts."""
    
    def __init__(self, identity: str = None, issuer: str = None):
        self.identity = identity or os.getenv("SIGSTORE_IDENTITY", "https://github.com/polymera-os")
        self.issuer = issuer or os.getenv("SIGSTORE_ISSUER", "https://token.actions.githubusercontent.com")
        
        # Check if cosign is available
        self.cosign_available = self._check_cosign()
        if not self.cosign_available:
            print("❌ cosign not found. Please install cosign first.")
            print("   Download from: https://github.com/sigstore/cosign/releases")
            sys.exit(1)
    
    def _check_cosign(self) -> bool:
        """Check if cosign is available in PATH."""
        try:
            result = subprocess.run(
                ["cosign", "version"],
                capture_output=True,
                text=True,
                check=True
            )
            print(f"✅ cosign found: {result.stdout.strip()}")
            return True
        except (subprocess.CalledProcessError, FileNotFoundError):
            return False
    
    def sign_artifact(self, artifact_path: Path, output_dir: Path = None) -> Dict[str, Any]:
        """Sign a single artifact using Sigstore."""
        
        if not artifact_path.exists():
            raise FileNotFoundError(f"Artifact not found: {artifact_path}")
        
        print(f"🔐 Signing artifact: {artifact_path.name}")
        
        # Determine output directory
        if output_dir is None:
            output_dir = artifact_path.parent / "signed"
        
        output_dir.mkdir(exist_ok=True)
        
        # Generate signature and certificate
        signature_file = output_dir / f"{artifact_path.name}.sig"
        certificate_file = output_dir / f"{artifact_path.name}.cert"
        
        try:
            # Sign the artifact
            result = subprocess.run([
                "cosign", "sign-blob",
                str(artifact_path),
                "--keyless",
                "--identity-token", self.identity,
                "--tlog-upload=true",
                "--output-signature", str(signature_file),
                "--output-certificate", str(certificate_file)
            ], capture_output=True, text=True, check=True)
            
            print(f"✅ Artifact signed successfully: {artifact_path.name}")
            
            # Calculate artifact hash
            artifact_hash = self._calculate_file_hash(artifact_path)
            
            # Return signing information
            return {
                "artifact": str(artifact_path),
                "signature_file": str(signature_file),
                "certificate_file": str(certificate_file),
                "hash": artifact_hash,
                "identity": self.identity,
                "issuer": self.issuer,
                "timestamp": datetime.datetime.utcnow().isoformat() + "Z",
                "status": "signed"
            }
            
        except subprocess.CalledProcessError as e:
            print(f"❌ Failed to sign {artifact_path.name}: {e}")
            print(f"Stderr: {e.stderr}")
            return {
                "artifact": str(artifact_path),
                "status": "failed",
                "error": str(e)
            }
    
    def sign_directory(self, directory_path: Path, output_dir: Path = None, 
                      file_patterns: List[str] = None) -> List[Dict[str, Any]]:
        """Sign all artifacts in a directory."""
        
        if not directory_path.exists():
            raise FileNotFoundError(f"Directory not found: {directory_path}")
        
        print(f"📁 Signing artifacts in directory: {directory_path}")
        
        # Default file patterns to sign
        if file_patterns is None:
            file_patterns = [
                "*.bin", "*.exe", "*.so", "*.dll", "*.dylib",  # Binaries
                "*.tar.gz", "*.zip", "*.deb", "*.rpm",  # Archives
                "*.json", "*.xml", "*.yaml", "*.yml",  # Configuration
                "*.md", "*.txt", "*.log"  # Documentation
            ]
        
        # Find all matching files
        artifacts = []
        for pattern in file_patterns:
            artifacts.extend(directory_path.glob(pattern))
        
        # Filter out already signed files
        artifacts = [a for a in artifacts if not a.name.endswith(('.sig', '.cert'))]
        
        print(f"🔍 Found {len(artifacts)} artifacts to sign")
        
        # Sign each artifact
        results = []
        for artifact in artifacts:
            try:
                result = self.sign_artifact(artifact, output_dir)
                results.append(result)
            except Exception as e:
                print(f"❌ Error signing {artifact}: {e}")
                results.append({
                    "artifact": str(artifact),
                    "status": "error",
                    "error": str(e)
                })
        
        return results
    
    def sign_sbom_files(self, sbom_dir: Path, output_dir: Path = None) -> List[Dict[str, Any]]:
        """Sign all SBOM files in a directory."""
        
        print(f"📋 Signing SBOM files in: {sbom_dir}")
        
        # Find all SBOM files
        sbom_patterns = ["*.json", "*.xml", "*.yaml", "*.yml"]
        artifacts = []
        
        for pattern in sbom_patterns:
            artifacts.extend(sbom_dir.glob(pattern))
        
        # Filter out already signed files
        artifacts = [a for a in artifacts if not a.name.endswith(('.sig', '.cert'))]
        
        print(f"🔍 Found {len(artifacts)} SBOM files to sign")
        
        # Sign each SBOM file
        results = []
        for artifact in artifacts:
            try:
                result = self.sign_artifact(artifact, output_dir)
                results.append(result)
            except Exception as e:
                print(f"❌ Error signing SBOM {artifact}: {e}")
                results.append({
                    "artifact": str(artifact),
                    "status": "error",
                    "error": str(e)
                })
        
        return results
    
    def verify_signature(self, artifact_path: Path, signature_file: Path, 
                        certificate_file: Path) -> bool:
        """Verify a signature using cosign."""
        
        if not all(p.exists() for p in [artifact_path, signature_file, certificate_file]):
            print(f"❌ Missing files for verification: {artifact_path}")
            return False
        
        try:
            result = subprocess.run([
                "cosign", "verify-blob",
                str(artifact_path),
                "--signature", str(signature_file),
                "--certificate", str(certificate_file),
                "--certificate-identity", self.identity,
                "--certificate-oidc-issuer", self.issuer
            ], capture_output=True, text=True, check=True)
            
            print(f"✅ Signature verified: {artifact_path.name}")
            return True
            
        except subprocess.CalledProcessError as e:
            print(f"❌ Signature verification failed: {artifact_path.name}")
            print(f"Stderr: {e.stderr}")
            return False
    
    def verify_directory(self, directory_path: Path) -> Dict[str, Any]:
        """Verify all signatures in a directory."""
        
        print(f"🔍 Verifying signatures in: {directory_path}")
        
        # Find all signature files
        signature_files = list(directory_path.glob("*.sig"))
        
        if not signature_files:
            print("⚠️  No signature files found")
            return {"verified": 0, "failed": 0, "total": 0}
        
        verified = 0
        failed = 0
        
        for sig_file in signature_files:
            # Determine artifact and certificate files
            artifact_file = sig_file.parent / sig_file.stem
            cert_file = sig_file.parent / f"{sig_file.stem}.cert"
            
            if artifact_file.exists() and cert_file.exists():
                if self.verify_signature(artifact_file, sig_file, cert_file):
                    verified += 1
                else:
                    failed += 1
            else:
                print(f"⚠️  Missing files for {sig_file.name}")
                failed += 1
        
        total = verified + failed
        
        print(f"📊 Verification results: {verified}/{total} verified, {failed} failed")
        
        return {
            "verified": verified,
            "failed": failed,
            "total": total
        }
    
    def _calculate_file_hash(self, file_path: Path) -> str:
        """Calculate SHA256 hash of a file."""
        sha256_hash = hashlib.sha256()
        with open(file_path, "rb") as f:
            for chunk in iter(lambda: f.read(4096), b""):
                sha256_hash.update(chunk)
        return sha256_hash.hexdigest()
    
    def generate_signing_report(self, results: List[Dict[str, Any]], 
                               output_file: Path = None) -> str:
        """Generate a comprehensive signing report."""
        
        if output_file is None:
            output_file = Path("signing-report.md")
        
        # Count results by status
        status_counts = {}
        for result in results:
            status = result.get("status", "unknown")
            status_counts[status] = status_counts.get(status, 0) + 1
        
        # Generate report
        report_lines = [
            "# Polymera OS Artifact Signing Report",
            f"Generated: {datetime.datetime.utcnow().strftime('%Y-%m-%d %H:%M:%S UTC')}",
            "",
            "## Signing Summary",
            "",
            f"- **Total Artifacts**: {len(results)}",
            f"- **Successfully Signed**: {status_counts.get('signed', 0)}",
            f"- **Failed**: {status_counts.get('failed', 0)}",
            f"- **Errors**: {status_counts.get('error', 0)}",
            "",
            "## Signing Configuration",
            "",
            f"- **Identity**: {self.identity}",
            f"- **Issuer**: {self.issuer}",
            f"- **Tool**: cosign",
            "",
            "## Detailed Results",
            ""
        ]
        
        # Group results by status
        for status in ["signed", "failed", "error"]:
            if status in status_counts:
                report_lines.extend([
                    f"### {status.title()} Artifacts",
                    ""
                ])
                
                for result in results:
                    if result.get("status") == status:
                        if status == "signed":
                            report_lines.extend([
                                f"- **{Path(result['artifact']).name}**",
                                f"  - Hash: `{result.get('hash', 'N/A')}`",
                                f"  - Signature: `{Path(result['signature_file']).name}`",
                                f"  - Certificate: `{Path(result['certificate_file']).name}`",
                                f"  - Timestamp: {result.get('timestamp', 'N/A')}",
                                ""
                            ])
                        else:
                            report_lines.extend([
                                f"- **{Path(result['artifact']).name}**",
                                f"  - Error: {result.get('error', 'Unknown error')}",
                                ""
                            ])
        
        report_lines.extend([
            "## Verification",
            "",
            "To verify signatures, use:",
            "```bash",
            f"cosign verify-blob <artifact> --signature <artifact>.sig --certificate <artifact>.cert",
            "--certificate-identity \"{self.identity}\"",
            f"--certificate-oidc-issuer \"{self.issuer}\"",
            "```",
            "",
            "---",
            "*Generated by Polymera OS Sigstore Signer*"
        ])
        
        report_content = "\n".join(report_lines)
        
        # Save report
        with open(output_file, 'w') as f:
            f.write(report_content)
        
        print(f"✅ Signing report saved to: {output_file}")
        return report_content
    
    def batch_sign(self, paths: List[Path], output_dir: Path = None) -> List[Dict[str, Any]]:
        """Sign multiple artifacts or directories in batch."""
        
        print(f"🚀 Starting batch signing of {len(paths)} paths...")
        
        all_results = []
        
        for path in paths:
            if path.is_file():
                # Sign individual file
                try:
                    result = self.sign_artifact(path, output_dir)
                    all_results.append(result)
                except Exception as e:
                    print(f"❌ Error signing file {path}: {e}")
                    all_results.append({
                        "artifact": str(path),
                        "status": "error",
                        "error": str(e)
                    })
            
            elif path.is_dir():
                # Sign directory contents
                try:
                    results = self.sign_directory(path, output_dir)
                    all_results.extend(results)
                except Exception as e:
                    print(f"❌ Error signing directory {path}: {e}")
                    all_results.append({
                        "artifact": str(path),
                        "status": "error",
                        "error": str(e)
                    })
            
            else:
                print(f"⚠️  Path does not exist: {path}")
        
        return all_results

def main():
    """Main entry point for the Sigstore signer."""
    
    parser = argparse.ArgumentParser(description="Sign artifacts using Sigstore keyless signing")
    parser.add_argument("paths", nargs="+", help="Paths to artifacts or directories to sign")
    parser.add_argument("--output-dir", "-o", help="Output directory for signed artifacts")
    parser.add_argument("--identity", "-i", help="Signing identity (default: SIGSTORE_IDENTITY env var)")
    parser.add_argument("--issuer", "-u", help="OIDC issuer (default: SIGSTORE_ISSUER env var)")
    parser.add_argument("--verify", "-v", action="store_true", help="Verify signatures after signing")
    parser.add_argument("--report", "-r", help="Generate signing report to specified file")
    parser.add_argument("--sbom-only", action="store_true", help="Only sign SBOM files")
    
    args = parser.parse_args()
    
    # Create signer
    signer = SigstoreSigner(identity=args.identity, issuer=args.issuer)
    
    try:
        # Convert paths to Path objects
        paths = [Path(p) for p in args.paths]
        
        # Determine output directory
        output_dir = Path(args.output_dir) if args.output_dir else None
        
        # Perform signing
        if args.sbom_only:
            # Only sign SBOM files
            results = []
            for path in paths:
                if path.is_dir():
                    results.extend(signer.sign_sbom_files(path, output_dir))
                else:
                    print(f"⚠️  Skipping non-directory path for SBOM signing: {path}")
        else:
            # Sign all artifacts
            results = signer.batch_sign(paths, output_dir)
        
        # Generate report if requested
        if args.report:
            signer.generate_signing_report(results, Path(args.report))
        else:
            # Generate default report
            signer.generate_signing_report(results)
        
        # Verify signatures if requested
        if args.verify:
            print("\n🔍 Verifying signatures...")
            for path in paths:
                if path.is_dir():
                    signer.verify_directory(path)
                else:
                    # For individual files, check if signature exists
                    sig_file = path.parent / f"{path.name}.sig"
                    cert_file = path.parent / f"{path.name}.cert"
                    if sig_file.exists() and cert_file.exists():
                        signer.verify_signature(path, sig_file, cert_file)
        
        # Print summary
        signed_count = sum(1 for r in results if r.get("status") == "signed")
        failed_count = sum(1 for r in results if r.get("status") in ["failed", "error"])
        
        print(f"\n🎉 Signing completed!")
        print(f"✅ Successfully signed: {signed_count}")
        print(f"❌ Failed: {failed_count}")
        
        if failed_count > 0:
            sys.exit(1)
        
    except Exception as e:
        print(f"❌ Error during signing: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
