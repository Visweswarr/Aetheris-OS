# Phase 4 Release Checklist

This checklist ensures a complete and secure release of Aetheris OS Phase 4.

## Pre-Release Preparation

### 1. Code Quality Verification
- [ ] **Run Phase 4 Verification**
  ```bash
  bash scripts/phase4-verify.sh
  ```
  - [ ] All critical tests pass
  - [ ] `artifacts/PHASE4_OK` exists
  - [ ] No critical failures in log

- [ ] **Code Formatting and Linting**
  ```bash
  make fmt && make lint
  ```
  - [ ] All code properly formatted
  - [ ] No linting errors
  - [ ] All warnings addressed

- [ ] **Test Suite**
  ```bash
  make test
  ```
  - [ ] All unit tests pass
  - [ ] Integration tests pass
  - [ ] Performance tests within limits

### 2. Documentation Review
- [ ] **Documentation Validation**
  ```bash
  make docs
  ```
  - [ ] All documentation links valid
  - [ ] Design tokens generated
  - [ ] API documentation up to date

- [ ] **Release Notes Preparation**
  - [ ] Review CHANGELOG.md
  - [ ] Update version numbers
  - [ ] Document breaking changes
  - [ ] List new features and improvements

### 3. Security Verification
- [ ] **Security Scan**
  ```bash
  # Run security scans
  cargo audit
  npm audit
  go mod audit
  ```
  - [ ] No critical vulnerabilities
  - [ ] All dependencies up to date
  - [ ] Security advisories reviewed

- [ ] **SBOM Generation**
  ```bash
  make sbom
  ```
  - [ ] Software Bill of Materials generated
  - [ ] All dependencies catalogued
  - [ ] SBOM files in `artifacts/sbom/`

## Version Bumping

### 4. Update Version Numbers
- [ ] **Update Cargo.toml**
  ```bash
  # Update version in services/*/Cargo.toml
  sed -i 's/version = "0.3.0"/version = "0.4.0"/g' services/*/Cargo.toml
  ```

- [ ] **Update package.json**
  ```bash
  # Update version in package.json
  npm version 0.4.0 --no-git-tag-version
  ```

- [ ] **Update Go modules**
  ```bash
  # Update version in go/tooling/*/go.mod
  find go/tooling -name "go.mod" -exec sed -i 's/v0.3.0/v0.4.0/g' {} \;
  ```

- [ ] **Update documentation**
  - [ ] Update version in `docs/phase-4/PHASE-4-OVERVIEW.md`
  - [ ] Update version in `design/Design-Tokens.yaml`
  - [ ] Update version in all README files

### 5. Commit Version Changes
```bash
git add .
git commit -m "chore(release): bump version to 0.4.0"
```

## Release Process

### 6. Create Release Branch
```bash
git checkout -b release/v0.4.0
git push origin release/v0.4.0
```

### 7. Final Verification
- [ ] **Run Complete Test Suite**
  ```bash
  bash scripts/phase4-verify.sh --clean
  ```
  - [ ] All tests pass
  - [ ] No regressions
  - [ ] Performance benchmarks met

- [ ] **Build All Artifacts**
  ```bash
  make package
  ```
  - [ ] All binaries build successfully
  - [ ] All packages created
  - [ ] Artifacts in `artifacts/` directory

### 8. Sign Artifacts
- [ ] **Generate Signatures**
  ```bash
  # Sign with minisign
  for file in artifacts/*.tar.gz artifacts/*.zip; do
    minisign -S -s keys/minisign.key -m "$file"
  done
  
  # Sign with cosign
  for file in artifacts/*.tar.gz artifacts/*.zip; do
    cosign sign-blob --key cosign.key "$file" > "$file.cosign"
  done
  ```

- [ ] **Verify Signatures**
  ```bash
  # Verify minisign signatures
  for file in artifacts/*.tar.gz artifacts/*.zip; do
    minisign -Vm "$file" -p keys/minisign.pub
  done
  
  # Verify cosign signatures
  for file in artifacts/*.tar.gz artifacts/*.zip; do
    cosign verify-blob --key cosign.pub --signature "$file.cosign" "$file"
  done
  ```

### 9. Create Git Tag
```bash
# Create annotated tag
git tag -a v0.4.0-phase4-ready -m "Release v0.4.0: Phase 4 Complete

🚀 Aetheris OS Phase 4 Release

## What's New
- Complete XR/Metaverse surface with multi-user persistence
- Comprehensive device runtime with AI capabilities
- Hardware Abstraction Layer (HAL) with Linux backends
- Edge AI perception and encoding pipelines
- Deterministic replay and cross-metaverse bridging

## Key Features
- XR Scene Management with DID avatars
- Device Runtime (Camera, Microphone, GPIO, ADC, Actuators)
- AI Pipelines (ONNX Vision, Whisper Audio)
- HAL with pluggable providers
- CapToken v2 authorization
- DAO governance integration
- NGFS content-addressed storage

## Verification
- All Phase 4 tests pass
- artifacts/PHASE4_OK exists
- Comprehensive test coverage
- Security scans clean
- SBOM generated

## Installation
See docs/phase-4/PHASE-4-OVERVIEW.md for installation instructions.

## Breaking Changes
- CapToken v2 format changes
- HAL provider interface updates
- XR scene format changes

## Contributors
- Core team
- Community contributors

## Verification Commands
\`\`\`bash
# Verify Phase 4 completion
if [ -f \"artifacts/PHASE4_OK\" ]; then
  echo \"✅ Phase 4 is complete!\"
else
  echo \"❌ Phase 4 verification pending\"
fi

# Run verification
bash scripts/phase4-verify.sh
\`\`\`

Signed-off-by: Aetheris OS Team <team@aetheris.io>"

# Push tag
git push origin v0.4.0-phase4-ready
```

### 10. GitHub Release
- [ ] **Create GitHub Release**
  - [ ] Go to GitHub Releases page
  - [ ] Click "Create a new release"
  - [ ] Select tag: `v0.4.0-phase4-ready`
  - [ ] Release title: `Aetheris OS v0.4.0 - Phase 4 Complete`
  - [ ] Copy release notes from CHANGELOG.md
  - [ ] Mark as latest release

- [ ] **Upload Artifacts**
  - [ ] Upload all binaries from `artifacts/`
  - [ ] Upload signature files (`*.sig`)
  - [ ] Upload SBOM files (`sbom-*.json`)
  - [ ] Upload checksums (`checksums.txt`)

### 11. Post-Release Tasks
- [ ] **Update Documentation**
  - [ ] Update installation guides
  - [ ] Update API documentation
  - [ ] Update community announcements

- [ ] **Announce Release**
  - [ ] Post on community forums
  - [ ] Update social media
  - [ ] Send release notifications
  - [ ] Update project website

- [ ] **Monitor Release**
  - [ ] Monitor download statistics
  - [ ] Watch for issue reports
  - [ ] Respond to community feedback
  - [ ] Track adoption metrics

## Release Artifacts

### Required Artifacts
- [ ] **Binaries**
  - [ ] `aetheris-os-0.4.0-linux-amd64.tar.gz`
  - [ ] `aetheris-os-0.4.0-linux-arm64.tar.gz`
  - [ ] `aetheris-os-0.4.0-windows-amd64.zip`
  - [ ] `aetheris-os-0.4.0-macos-amd64.tar.gz`
  - [ ] `aetheris-os-0.4.0-macos-arm64.tar.gz`

- [ ] **Signatures**
  - [ ] `*.sig` files (minisign format)
  - [ ] `*.cosign` files (cosign format)

- [ ] **SBOM**
  - [ ] `sbom-0.4.0-spdx.json`
  - [ ] `sbom-0.4.0-cyclonedx.json`

- [ ] **Checksums**
  - [ ] `checksums.txt` with SHA256 hashes

### Optional Artifacts
- [ ] **Docker Images**
  - [ ] `aetheris/os:0.4.0`
  - [ ] `aetheris/os:0.4.0-alpine`
  - [ ] `aetheris/os:latest`

- [ ] **Package Managers**
  - [ ] NPM package
  - [ ] Cargo crate
  - [ ] Go module

## Verification Commands

### Verify Release Integrity
```bash
# Verify Phase 4 completion
if [ -f "artifacts/PHASE4_OK" ]; then
  echo "✅ Phase 4 is complete!"
  cat artifacts/PHASE4_OK
else
  echo "❌ Phase 4 verification pending"
  exit 1
fi

# Verify signatures
for file in artifacts/*.tar.gz artifacts/*.zip; do
  echo "Verifying $file..."
  minisign -Vm "$file" -p keys/minisign.pub
  cosign verify-blob --key cosign.pub --signature "$file.cosign" "$file"
done

# Verify SBOM
syft attest --key cosign.key artifacts/aetheris-os-0.4.0-linux-amd64.tar.gz

# Verify checksums
sha256sum -c checksums.txt
```

### Test Installation
```bash
# Test Docker installation
docker pull aetheris/os:0.4.0
docker run --rm aetheris/os:0.4.0 --version

# Test binary installation
tar -xzf aetheris-os-0.4.0-linux-amd64.tar.gz
./aetheris-os --version
./aetheris-os --help
```

## Rollback Plan

If issues are discovered after release:

1. **Immediate Actions**
   - [ ] Mark release as deprecated
   - [ ] Update release notes with known issues
   - [ ] Provide workarounds if possible

2. **Hotfix Process**
   - [ ] Create hotfix branch from previous stable release
   - [ ] Apply minimal fix
   - [ ] Run full verification
   - [ ] Create patch release (0.4.1)

3. **Communication**
   - [ ] Notify community of issues
   - [ ] Provide timeline for fix
   - [ ] Update documentation

## Success Criteria

Release is considered successful when:

- [ ] All Phase 4 tests pass
- [ ] `artifacts/PHASE4_OK` exists
- [ ] All artifacts are signed and verified
- [ ] SBOM is generated and attached
- [ ] GitHub release is published
- [ ] Community can successfully install and use
- [ ] No critical issues reported within 24 hours

## Emergency Contacts

- **Release Manager**: [Name] <email>
- **Security Team**: [Name] <email>
- **Community Manager**: [Name] <email>
- **Technical Lead**: [Name] <email>

## Release Notes Template

```markdown
# Aetheris OS v0.4.0 - Phase 4 Complete

## 🚀 What's New

### XR/Metaverse Surface
- Multi-user persistence with NGFS snapshots
- Session recording/replay with deterministic execution
- Cross-metaverse bridge (Godot ↔ WebXR/IPFS)
- DAO-governed scene editing with CapToken v2

### Device Runtime
- Camera & Microphone capture with V4L2/ALSA
- BLE device management and sensor framework
- GPIO, ADC, and actuator control
- Hardware Abstraction Layer (HAL) with Linux backends

### Edge AI
- ONNX Runtime for object detection
- Whisper.cpp for speech recognition
- Deterministic AI inference with replay
- CBOR event schemas for AI results

## 🔧 Technical Improvements
- CapToken v2 authorization system
- DAO governance integration
- NGFS content-addressed storage
- Comprehensive test coverage
- Security hardening

## 📦 Installation

### Docker
```bash
docker pull aetheris/os:0.4.0
```

### Binary
Download from [GitHub Releases](https://github.com/aetheris/os/releases/tag/v0.4.0-phase4-ready)

## 🔍 Verification

All releases are signed and include SBOM:
- Signatures: `*.sig` files (minisign format)
- SBOM: `sbom-*.json` (SPDX format)
- Checksums: `checksums.txt`

## 🚨 Breaking Changes
- CapToken v2 format changes
- HAL provider interface updates
- XR scene format changes

## 📚 Documentation
- [Installation Guide](https://docs.aetheris.io/installation)
- [API Reference](https://docs.aetheris.io/api)
- [Phase 4 Overview](https://docs.aetheris.io/phase-4)

## 🤝 Contributors
Thanks to all contributors who made this release possible!

## 🔗 Links
- [GitHub Repository](https://github.com/aetheris/os)
- [Documentation](https://docs.aetheris.io)
- [Community](https://community.aetheris.io)
```

---

**Remember**: This checklist ensures a complete, secure, and professional release of Aetheris OS Phase 4. Follow each step carefully and verify completion before proceeding to the next step.
