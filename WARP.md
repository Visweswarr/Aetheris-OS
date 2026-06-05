# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

Project context
- Polymera OS is a polyglot monorepo (Rust, Go, Python, TypeScript, C) organized as a microkernel + services + runtime + UI stack. Bazel configs exist for legacy builds, but day-to-day development is driven by Makefile wrappers and language-native tooling.
- CI enforces formatting, linting, SBOM generation, and Sigstore signing across languages.

Prerequisites (from README.md)
- Rust ≥1.75, Go ≥1.21, Node.js ≥18, Python ≥3.11; Docker optional
- Windows: PowerShell (pwsh) supported; bootstrap script available

Core commands
- Bootstrap environment
  - POSIX shells: make bootstrap
  - Windows: pwsh -ExecutionPolicy Bypass -File scripts/bootstrap-windows.ps1 (the Makefile will auto-detect when possible)

- Format all languages
  - make fmt
  - Language-specific (if needed): make fmt-rust | fmt-go | fmt-ts | fmt-py

- Lint all languages
  - make lint
  - Language-specific (if needed): make lint-rust | lint-go | lint-ts | lint-py

- Test matrix (Rust, Go, TypeScript, Python)
  - make test
  - Language-specific (use these when iterating):
    - Rust (workspace-wide): cargo test --workspace
    - Rust (single package/test): cargo test -p <crate> <pattern>
      - Example: (services/posix) cargo test --release integration
    - Go (module dir): go test ./...
    - Go (single test): go test -run "^TestName$" ./path/to/pkg
    - Python (tooling/python): pytest -m "not hw"
    - Python (single test): pytest -k "name_pattern"
    - TypeScript (typecheck/lint used in CI):
      - npx tsc --noEmit
      - npx eslint "**/*.{ts,tsx}" --max-warnings 0
      - If a package defines tests (e.g., tooling/ts or ui/*), run npm test there

- Build artifacts
  - Rust (component dir): cargo build --release
  - Go (component dir, e.g., go/tooling/xrctl): go build -v -o <out> .
  - TypeScript UI (ui/* when present): npm ci && npm run build

- Package CLI tools (aggregated)
  - make package (produces artifacts in artifacts/pkg/)

- Documentation validation and token generation
  - make docs (markdown link checks + design tokens via scripts/generate-tokens.ts)

- SBOM generation
  - make sbom (wrapper around tools/sbom/generate.sh; CI also generates/merges SBOMs and signs them)

- Clean
  - make clean

Verification and smoke flows (from README.md)
- Phase 4 verification: bash scripts/phase4-verify.sh
- Smoke tests: bash scripts/p4-contracts-smoke.sh; bash scripts/p4-xr-determinism.sh; bash scripts/p4-hal-toggle.sh

High-level architecture (big picture)
- Stack (from README.md): Applications (WASI/CPython/JVM/CLR) → Runtime Layer (WASI host, language bridges) → Service Layer (DeviceKit, PolyAudio, PolyNet, KeyVault, NGFS, Attestation, Wallet, etc.) → Kernel (PolymeraCore, PolyBus, PolyMemory) → Hardware Abstraction (secure boot, TPM, RNG)
- Organization (important top-levels):
  - kernel/: microkernel primitives (scheduling, memory, messaging)
  - services/: major OS services (e.g., DeviceKit, PolyAudio, PolyNet, NGFS, KeyVault, POSIX service)
  - runtime/: WASI host and bridges for language runtimes
  - ui/: compositor/UI surfaces; e.g., ui/posix checked in CI for typecheck/lint
  - go/tooling/: CLI utilities (xrctl, devctl, ai_core); each subdir is a Go module
  - tooling/python/: validators, performance and capability checks (pytest used with -m "not hw")
  - infra/.github/workflows/: CI orchestrates linting, building, SBOM/signing, performance gates
  - WORKSPACE and BUILD files: Bazel legacy; not required for routine dev, but CI can verify bazel build

What CI enforces (from .github/workflows/*.yml)
- Linting/formatting gates: rustfmt + clippy (Rust); black + ruff (Python); prettier + eslint + tsc (TS/JS); shfmt + shellcheck (shell); markdownlint + JSON/YAML validation (docs)
- Conventional commits and governance: commitlint, CODEOWNERS coverage, license headers
- Build verification: language-native builds; Bazel build //... is also validated
- Supply chain: SBOM generation and merging (Syft, cargo-sbom, license checkers) and Sigstore signing/verification for artifacts and SBOMs

Language-specific notes
- Rust: Multi-crate workspace; common locations include services/*, security/*, tests/*; use cargo test/build within component dirs; clippy must pass with -D warnings in CI
- Go: go/tooling/* modules (xrctl, devctl, ai_core); use go test/build in each module; CI caches modules and runs module-local builds
- Python: tooling/python for validators/tests; pytest markers exclude hardware with -m "not hw"; black/ruff formatting required
- TypeScript/UI: Type checks and linting enforced; build steps run when ui/* defines a package.json

Windows-specific
- Use PowerShell 7+ (pwsh). For initial setup, prefer: pwsh -ExecutionPolicy Bypass -File scripts/bootstrap-windows.ps1
- Most repo tasks are wrapped by make; where Makefile uses POSIX tooling, run the underlying language-native commands directly in component directories as listed above
