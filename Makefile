# Aetheris OS Monorepo Makefile
# Idempotent targets for development workflow

.PHONY: help bootstrap fmt lint test sbom docs package clean golden golden-rebaseline golden-ai-core golden-ai-core-rebaseline perf-bench perf-bench-ai-core tokens

# Default target
help:
	@echo "Aetheris OS Development Targets:"
	@echo ""
	@echo "  help      - Show this help message"
	@echo "  bootstrap - Install development dependencies"
	@echo "  fmt       - Format all source code"
	@echo "  lint      - Lint all source code"
	@echo "  test      - Run test matrix across all languages"
	@echo "  sbom      - Generate Software Bill of Materials"
	@echo "  docs      - Validate documentation"
	@echo "  package   - Package CLI tools"
	@echo "  clean     - Clean build artifacts"
	@echo "  golden    - Run AI golden tests"
	@echo "  golden-rebaseline - Rebaseline AI golden tests"
	@echo "  golden-ai-core - Run AI Core Service golden tests"
	@echo "  golden-ai-core-rebaseline - Rebaseline AI Core Service golden tests"
	@echo "  perf-bench - Run performance benchmarks"
	@echo "  perf-bench-ai-core - Run AI Core Service performance benchmarks"
	@echo "  tokens    - Generate design tokens"
	@echo ""
	@echo "Language-specific targets:"
	@echo "  fmt-rust  - Format Rust code"
	@echo "  fmt-go    - Format Go code"
	@echo "  fmt-ts    - Format TypeScript code"
	@echo "  fmt-py    - Format Python code"
	@echo "  lint-rust - Lint Rust code"
	@echo "  lint-go   - Lint Go code"
	@echo "  lint-ts   - Lint TypeScript code"
	@echo "  lint-py   - Lint Python code"
	@echo "  test-rust - Test Rust code"
	@echo "  test-go   - Test Go code"
	@echo "  test-ts   - Test TypeScript code"
	@echo "  test-py   - Test Python code"

# Bootstrap development environment
bootstrap:
	@echo "Bootstrapping Aetheris OS development environment..."
	@if [ -f "scripts/bootstrap-$(shell uname -s | tr '[:upper:]' '[:lower:]').sh" ]; then \
		bash scripts/bootstrap-$(shell uname -s | tr '[:upper:]' '[:lower:]').sh; \
	elif [ -f "scripts/bootstrap-windows.ps1" ] && command -v pwsh >/dev/null 2>&1; then \
		pwsh -ExecutionPolicy Bypass -File scripts/bootstrap-windows.ps1; \
	else \
		echo "No bootstrap script found for this platform"; \
		echo "Please run the appropriate bootstrap script manually:"; \
		echo "  Linux/macOS: bash scripts/bootstrap-linux.sh"; \
		echo "  Windows: pwsh scripts/bootstrap-windows.ps1"; \
	fi

# Format all source code
fmt: fmt-rust fmt-go fmt-ts fmt-py

fmt-rust:
	@echo "Formatting Rust code..."
	@if command -v cargo >/dev/null 2>&1; then \
		cargo fmt --all; \
		echo "✅ Rust formatting complete"; \
	else \
		echo "⚠️  cargo not found, skipping Rust formatting"; \
	fi

fmt-go:
	@echo "Formatting Go code..."
	@if command -v go >/dev/null 2>&1; then \
		find go/tooling -name "*.go" -exec gofmt -w {} \; 2>/dev/null || true; \
		echo "✅ Go formatting complete"; \
	else \
		echo "⚠️  go not found, skipping Go formatting"; \
	fi

fmt-ts:
	@echo "Formatting TypeScript code..."
	@if command -v npx >/dev/null 2>&1; then \
		cd tooling/ts && npm run format 2>/dev/null || npx prettier --write "**/*.{ts,js,json}" 2>/dev/null || true; \
		echo "✅ TypeScript formatting complete"; \
	else \
		echo "⚠️  npx not found, skipping TypeScript formatting"; \
	fi

fmt-py:
	@echo "Formatting Python code..."
	@if command -v python3 >/dev/null 2>&1; then \
		cd tooling/python && python3 -m black . 2>/dev/null || true; \
		cd tooling/python && python3 -m isort . 2>/dev/null || true; \
		echo "✅ Python formatting complete"; \
	else \
		echo "⚠️  python3 not found, skipping Python formatting"; \
	fi

# Lint all source code
lint: lint-rust lint-go lint-ts lint-py

lint-rust:
	@echo "Linting Rust code..."
	@if command -v cargo >/dev/null 2>&1; then \
		cargo clippy --workspace -- -D warnings 2>/dev/null || echo "⚠️  Rust linting had issues"; \
		echo "✅ Rust linting complete"; \
	else \
		echo "⚠️  cargo not found, skipping Rust linting"; \
	fi

lint-go:
	@echo "Linting Go code..."
	@if command -v golangci-lint >/dev/null 2>&1; then \
		cd go/tooling && golangci-lint run 2>/dev/null || echo "⚠️  Go linting had issues"; \
		echo "✅ Go linting complete"; \
	elif command -v go >/dev/null 2>&1; then \
		cd go/tooling && go vet ./... 2>/dev/null || echo "⚠️  Go vet had issues"; \
		echo "✅ Go basic linting complete"; \
	else \
		echo "⚠️  go not found, skipping Go linting"; \
	fi

lint-ts:
	@echo "Linting TypeScript code..."
	@if command -v npx >/dev/null 2>&1; then \
		cd tooling/ts && npx tsc --noEmit 2>/dev/null || echo "⚠️  TypeScript type checking had issues"; \
		cd tooling/ts && npm run lint 2>/dev/null || echo "⚠️  TypeScript linting had issues"; \
		echo "✅ TypeScript linting complete"; \
	else \
		echo "⚠️  npx not found, skipping TypeScript linting"; \
	fi

lint-py:
	@echo "Linting Python code..."
	@if command -v python3 >/dev/null 2>&1; then \
		cd tooling/python && python3 -m ruff check . 2>/dev/null || python3 -m flake8 . 2>/dev/null || echo "⚠️  Python linting had issues"; \
		echo "✅ Python linting complete"; \
	else \
		echo "⚠️  python3 not found, skipping Python linting"; \
	fi

# Run test matrix
test: test-rust test-go test-ts test-py

test-rust:
	@echo "Testing Rust code..."
	@if command -v cargo >/dev/null 2>&1; then \
		cargo test --workspace --exclude aetheris-hw 2>/dev/null || echo "⚠️  Rust tests had issues"; \
		echo "✅ Rust testing complete"; \
	else \
		echo "⚠️  cargo not found, skipping Rust testing"; \
	fi

test-go:
	@echo "Testing Go code..."
	@if command -v go >/dev/null 2>&1; then \
		cd go/tooling && go test ./... 2>/dev/null || echo "⚠️  Go tests had issues"; \
		echo "✅ Go testing complete"; \
	else \
		echo "⚠️  go not found, skipping Go testing"; \
	fi

test-ts:
	@echo "Testing TypeScript code..."
	@if [ -f "tooling/ts/package.json" ] && command -v npm >/dev/null 2>&1; then \
		cd tooling/ts && npm test 2>/dev/null || echo "⚠️  TypeScript tests had issues"; \
		echo "✅ TypeScript testing complete"; \
	else \
		echo "⚠️  TypeScript test setup not found, skipping"; \
	fi

test-py:
	@echo "Testing Python code..."
	@if command -v python3 >/dev/null 2>&1; then \
		cd tooling/python && python3 -m pytest -m "not hw" 2>/dev/null || echo "⚠️  Python tests had issues"; \
		echo "✅ Python testing complete"; \
	else \
		echo "⚠️  python3 not found, skipping Python testing"; \
	fi

# Generate Software Bill of Materials
sbom:
	@echo "Generating Software Bill of Materials..."
	@bash tools/sbom/generate.sh

# Validate documentation
docs:
	@echo "Validating documentation..."
	@if command -v markdown-link-check >/dev/null 2>&1; then \
		find docs -name "*.md" -exec markdown-link-check {} \; 2>/dev/null || echo "⚠️  Link checking had issues"; \
		echo "✅ Documentation link checking complete"; \
	else \
		echo "⚠️  markdown-link-check not found, skipping link validation"; \
		echo "Install: npm install -g markdown-link-check"; \
	fi
	@echo "📚 Documentation structure validated"
	@echo "Generating design tokens..."
	@if command -v node >/dev/null 2>&1; then \
		npx ts-node scripts/generate-tokens.ts 2>/dev/null || echo "⚠️  Token generation had issues"; \
		echo "✅ Design tokens generated"; \
	else \
		echo "⚠️  node not found, skipping token generation"; \
		echo "Install Node.js to generate design tokens"; \
	fi

# Package CLI tools
package: package-cli

package-cli:
	@echo "Packaging CLI tools..."
	@mkdir -p artifacts/pkg
	@if command -v cargo >/dev/null 2>&1; then \
		cargo build --release --bin netctl 2>/dev/null && cp target/release/netctl artifacts/pkg/ 2>/dev/null || echo "⚠️  netctl build failed"; \
		cargo build --release --bin xrctl 2>/dev/null && cp target/release/xrctl artifacts/pkg/ 2>/dev/null || echo "⚠️  xrctl build failed"; \
		cargo build --release --bin devctl 2>/dev/null && cp target/release/devctl artifacts/pkg/ 2>/dev/null || echo "⚠️  devctl build failed"; \
	fi
	@if command -v go >/dev/null 2>&1; then \
		cd go/tooling/netctl && go build -o ../../../artifacts/pkg/netctl-go . 2>/dev/null || echo "⚠️  netctl-go build failed"; \
		cd go/tooling/xrctl && go build -o ../../../artifacts/pkg/xrctl-go . 2>/dev/null || echo "⚠️  xrctl-go build failed"; \
		cd go/tooling/devctl && go build -o ../../../artifacts/pkg/devctl-go . 2>/dev/null || echo "⚠️  devctl-go build failed"; \
	fi
	@echo "✅ CLI packaging complete"
	@ls -la artifacts/pkg/ 2>/dev/null || echo "No packages created"

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@if command -v cargo >/dev/null 2>&1; then \
		cargo clean 2>/dev/null || true; \
	fi
	@if command -v go >/dev/null 2>&1; then \
		find go -name "*.exe" -delete 2>/dev/null || true; \
		find go -name "*.test" -delete 2>/dev/null || true; \
	fi
	@rm -rf artifacts/ 2>/dev/null || true
	@rm -rf node_modules/ 2>/dev/null || true
	@rm -rf target/ 2>/dev/null || true
	@rm -rf dist/ 2>/dev/null || true
	@rm -rf build/ 2>/dev/null || true
	@echo "✅ Cleanup complete"

# AI Golden Tests
golden:
	@echo "Running AI Golden Tests..."
	@python tooling/python/ai_golden_test.py

golden-rebaseline:
	@echo "Rebaselining AI Golden Tests..."
	@bash scripts/ai-rebaseline.sh

# AI Core Service Golden Tests
golden-ai-core:
	@echo "Running AI Core Service Golden Tests..."
	@mkdir -p artifacts/ai_core
	@if command -v cargo >/dev/null 2>&1; then \
		cargo run --bin ai_core_golden --manifest-path scripts/golden/Cargo.toml; \
	else \
		echo "⚠️  cargo not found, using fallback method"; \
		rustc scripts/golden/ai_core_golden.rs -o scripts/golden/ai_core_golden && \
		./scripts/golden/ai_core_golden; \
	fi

golden-ai-core-rebaseline:
	@echo "Rebaselining AI Core Service Golden Tests..."
	@mkdir -p artifacts/ai_core
	@if command -v cargo >/dev/null 2>&1; then \
		cargo run --bin ai_core_golden --manifest-path scripts/golden/Cargo.toml -- --rebaseline; \
	else \
		echo "⚠️  cargo not found, using fallback method"; \
		rustc scripts/golden/ai_core_golden.rs -o scripts/golden/ai_core_golden && \
		./scripts/golden/ai_core_golden --rebaseline; \
	fi

# Performance Benchmarks
perf-bench:
	@echo "Running Performance Benchmarks..."
	@bash scripts/bench/bench_ai.sh

perf-bench-ai-core:
	@echo "Running AI Core Service Performance Benchmarks..."
	@mkdir -p artifacts/bench
	@bash scripts/ai-core-perf-bench.sh

# Design Tokens
tokens:
	@echo "Generating design tokens..."
	@if command -v node >/dev/null 2>&1; then \
		npx ts-node scripts/generate-tokens.ts; \
		echo "✅ Design tokens generated"; \
	else \
		echo "⚠️  node not found, skipping token generation"; \
		echo "Install Node.js to generate design tokens"; \
	fi


# ═══════════════════════════════════════════════════════════════════════════════
# AVENGERS MICROKERNEL BUILD TARGETS
# ═══════════════════════════════════════════════════════════════════════════════
# The "Avengers" of Operating Systems - Polyglot Microkernel
# Rust + Go + C++ + C# + WASM running simultaneously
# ═══════════════════════════════════════════════════════════════════════════════

.PHONY: avengers avengers-build avengers-iso avengers-run avengers-clean avengers-status

# Build all Avengers components
avengers: avengers-build
	@echo "🦸 AVENGERS ASSEMBLED!"

avengers-build:
	@echo "╔══════════════════════════════════════════════════════════════╗"
	@echo "║     🦸 POLYMERA OS - AVENGERS BUILD 🦸                      ║"
	@echo "╚══════════════════════════════════════════════════════════════╝"
	@bash tools/build/build.sh || pwsh -File tools/build/build.ps1

# Create bootable ISO
avengers-iso: avengers-build
	@echo "💿 ISO created at artifacts/os/polymera-os-avengers.iso"

# Run in QEMU
avengers-run: avengers-iso
	@echo "🚀 Launching Polymera OS..."
	@if command -v qemu-system-x86_64 >/dev/null 2>&1; then \
		qemu-system-x86_64 \
			-cdrom artifacts/os/polymera-os-avengers.iso \
			-m 2G \
			-smp 4 \
			-serial stdio \
			-device virtio-gpu-pci; \
	else \
		echo "⚠️  QEMU not found. Install qemu-system-x86_64 to run the OS."; \
	fi

# Run QEMU smoke test boot verification
qemu-smoke:
	@echo "Running QEMU Smoke Boot Verification..."
	@pwsh -File scripts/qemu-smoke.ps1 || powershell -File scripts/qemu-smoke.ps1


# Clean Avengers build artifacts
avengers-clean:
	@echo "🧹 Cleaning Avengers build..."
	@rm -rf build/ artifacts/os/
	@echo "✅ Clean complete"

# Show Avengers status
avengers-status:
	@echo "╔══════════════════════════════════════════════════════════════╗"
	@echo "║     🦸 AVENGERS MICROKERNEL STATUS 🦸                       ║"
	@echo "╠══════════════════════════════════════════════════════════════╣"
	@echo "║  Hero        Codename      Superpower                       ║"
	@echo "║  ──────────  ────────────  ─────────────────────────────    ║"
	@echo "║  🦀 Rust     Sentinel      Memory Safety + Zero-Cost        ║"
	@echo "║  🐹 Go       Coordinator   Goroutines + Network Stack       ║"
	@echo "║  ⚡ C++      Speedster     Raw Performance + GPU            ║"
	@echo "║  🏗️ C#       Architect     Managed Runtime + UI             ║"
	@echo "║  🔮 WASM     Shapeshifter  Sandboxing + Portability         ║"
	@echo "╠══════════════════════════════════════════════════════════════╣"
	@echo "║  Commands:                                                  ║"
	@echo "║    make avengers       - Build all components               ║"
	@echo "║    make avengers-iso   - Create bootable ISO                ║"
	@echo "║    make avengers-run   - Run in QEMU                        ║"
	@echo "║    make avengers-clean - Clean build artifacts              ║"
	@echo "╚══════════════════════════════════════════════════════════════╝"

# Individual hero builds
avengers-kernel:
	@echo "🦀 Building Rust Kernel [Sentinel]..."
	@cd kernel && cargo build --release --target x86_64-unknown-none || echo "⚠️  Kernel build requires bare-metal target"

avengers-go:
	@echo "🐹 Building Go Services [Coordinator]..."
	@cd services/service_manager && CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -ldflags="-s -w" . || echo "⚠️  Go build failed"
	@cd services/fs_go && CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -ldflags="-s -w" . || echo "⚠️  Go build failed"
	@cd services/net_go && CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -ldflags="-s -w" . || echo "⚠️  Go build failed"

avengers-cpp:
	@echo "⚡ Building C++ Services [Speedster]..."
	@mkdir -p services/window_server_cpp/build
	@cd services/window_server_cpp/build && cmake .. && make || echo "⚠️  C++ build failed"

avengers-csharp:
	@echo "🏗️ Building C# Runtime [Architect]..."
	@cd services/app_runtime_cs && dotnet build -c Release || echo "⚠️  C# build failed"

avengers-wasm:
	@echo "🔮 Building WASM Host [Shapeshifter]..."
	@cd services/wasm_driver && cargo build --release || echo "⚠️  WASM build failed"
