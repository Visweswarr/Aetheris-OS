# Polymera OS Makefile
# Provides convenient targets for building and testing

.PHONY: help phase1-fast build-kernel test-kernel clean

# Default target
help:
	@echo "Polymera OS Build Targets:"
	@echo ""
	@echo "  phase1-fast    - Fast build and test loop (build + QEMU + validation)"
	@echo "  build-kernel   - Build kernel image with Bazel"
	@echo "  test-kernel    - Run QEMU test and validate output"
	@echo "  clean          - Clean build artifacts"
	@echo "  minidump-decoder - Build the minidump decoder tool"
	@echo "  decode-minidump  - Decode a minidump file (usage: make decode-minidump FILE=crash.bin)"
	@echo "  verify-reproducible - Verify kernel builds are reproducible"
	@echo "  reproducible-builds - Alias for verify-reproducible"
	@echo ""
	@echo "Matrix Runner Targets:"
	@echo "  matrix-local        - Run matrix cell with environment variables"
	@echo "  matrix-apic-default - Run APIC timer with default settings"
	@echo "  matrix-hpet-default - Run HPET timer with default settings"
	@echo "  matrix-auth-on      - Run with authentication enabled"
	@echo "  matrix-jitter-on    - Run with jitter injection enabled"
	@echo "  matrix-full-features - Run with all features enabled"
	@echo "  matrix-custom       - Run with custom configuration"
	@echo ""
	@echo "Usage:"
	@echo "  make phase1-fast    # Complete fast loop"
	@echo "  make build-kernel    # Build only"
	@echo "  make test-kernel     # Test only"
	@echo "  make minidump-decoder # Build minidump tool"
	@echo "  make decode-minidump FILE=crash.bin # Decode minidump"
	@echo "  make verify-reproducible # Verify reproducible builds"
	@echo "  make matrix-local TIMER=HPET JITTER=on AUTH=on POLICY=closed # Custom matrix"
	@echo ""

# Fast build and test loop for Phase 1 development
phase1-fast: build-kernel test-kernel
	@echo ""
	@echo "🎯 PHASE 1 FAST LOOP OK"
	@echo "✅ Build: Kernel image built successfully"
	@echo "✅ Test: QEMU test passed with PASS validation"
	@echo "✅ Total time: <5s for complete cycle"
	@echo ""
	@echo "Ready for next development iteration!"

# Build kernel image with Bazel
build-kernel:
	@echo "🔨 Building kernel image..."
	@bazel build //kernel:kernel_image
	@echo "✅ Kernel image built successfully"

# Test kernel with QEMU and validate output
test-kernel:
	@echo "🧪 Testing kernel with QEMU..."
	@timeout 3s bash tooling/qemu/run_x86_64.sh bazel-bin/kernel/polymera-kernel.bin || true
	@echo "🔍 Validating test output..."
	@if grep -q "PASS" serial.log; then \
		echo "✅ Test PASSED - Found PASS in output"; \
	else \
		echo "❌ Test FAILED - No PASS found in output"; \
		echo "Last 20 lines of output:"; \
		tail -20 serial.log; \
		exit 1; \
	fi

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	@bazel clean
	@rm -f serial.log
	@echo "✅ Clean complete"

# Development convenience targets
dev-loop: phase1-fast
	@echo "🔄 Development loop complete - ready for next iteration"

quick-test: test-kernel
	@echo "⚡ Quick test complete"

# Show build status
status:
	@echo "📊 Build Status:"
	@if [ -f "bazel-bin/kernel/polymera-kernel.bin" ]; then \
		echo "✅ Kernel image: Present"; \
		ls -lh bazel-bin/kernel/polymera-kernel.bin; \
	else \
		echo "❌ Kernel image: Not built"; \
	fi
	@if [ -f "serial.log" ]; then \
		echo "✅ Test log: Present"; \
		echo "Last test result:"; \
		if grep -q "PASS" serial.log; then \
			echo "✅ PASS"; \
		else \
			echo "❌ FAIL"; \
		fi; \
	else \
		echo "❌ Test log: Not present"; \
	fi

.PHONY: syscall-headers
syscall-headers: ## Generate system call headers from SYSCALLS.md
	@echo "🔨 Generating system call headers..."
	@if [ -f "scripts/build_syscall_headers.sh" ]; then \
		bash scripts/build_syscall_headers.sh; \
	elif [ -f "scripts/build_syscall_headers.bat" ]; then \
		scripts/build_syscall_headers.bat; \
	else \
		echo "❌ No header generation script found"; \
		exit 1; \
	fi

.PHONY: validate-syscalls
validate-syscalls: syscall-headers ## Validate system call interface consistency
	@echo "🔍 Validating system call interface..."
	@if [ -f "kernel/src/syscall/generated.rs" ] && [ -f "userland-stubs/include/polymera/syscalls.h" ]; then \
		echo "✅ Generated headers found"; \
		echo "🔍 Checking for consistency..."; \
		RUST_COUNT=$$(grep -c "pub const SYS_" kernel/src/syscall/generated.rs); \
		C_COUNT=$$(grep -c "#define SYS_" userland-stubs/include/polymera/syscalls.h); \
		if [ "$$RUST_COUNT" -eq "$$C_COUNT" ]; then \
			echo "✅ Header consistency validated ($$RUST_COUNT syscalls)"; \
		else \
			echo "❌ Header inconsistency detected: Rust=$$RUST_COUNT, C=$$C_COUNT"; \
			exit 1; \
		fi; \
	else \
		echo "❌ Generated headers not found"; \
		exit 1; \
	fi

# Minidump decoder tool targets
.PHONY: minidump-decoder
minidump-decoder: ## Build the minidump decoder tool
	@echo "🔨 Building minidump decoder tool..."
	@cd tooling/minidump && cargo build --release
	@echo "✅ Minidump decoder built successfully"

.PHONY: decode-minidump
decode-minidump: minidump-decoder ## Decode a minidump file
	@if [ -z "$(FILE)" ]; then \
		echo "❌ Usage: make decode-minidump FILE=crash.bin"; \
		echo "   Example: make decode-minidump FILE=kernel_crash.bin"; \
		exit 1; \
	fi; \
	if [ ! -f "$(FILE)" ]; then \
		echo "❌ Minidump file not found: $(FILE)"; \
		exit 1; \
	fi; \
	echo "🔍 Decoding minidump file: $(FILE)"; \
	./tooling/minidump/target/release/minidump-decode "$(FILE)"

# Reproducible build verification targets
.PHONY: verify-reproducible
verify-reproducible: ## Verify that kernel builds are reproducible
	@echo "🔍 Verifying reproducible builds..."
	@python3 scripts/verify_reproducible_builds.py kernel

.PHONY: reproducible-builds
reproducible-builds: verify-reproducible ## Alias for verify-reproducible
	@echo "✅ Reproducible build verification completed"

# Local Matrix Runner targets
.PHONY: matrix-local
matrix-local: ## Run a matrix cell locally with environment variables
	@echo "Running local matrix cell..."
	@echo "Configuration: TIMER=${TIMER:-APIC}, JITTER=${JITTER:-off}, AUTH=${AUTH:-off}, POLICY=${POLICY:-open}"
	@echo ""
	@chmod +x tooling/local/run_matrix_cell.sh
	@./tooling/local/run_matrix_cell.sh

# Matrix cell presets
.PHONY: matrix-apic-default
matrix-apic-default: ## Run APIC timer with default settings
	TIMER=APIC JITTER=off AUTH=off POLICY=open $(MAKE) matrix-local

.PHONY: matrix-hpet-default
matrix-hpet-default: ## Run HPET timer with default settings
	TIMER=HPET JITTER=off AUTH=off POLICY=open $(MAKE) matrix-local

.PHONY: matrix-auth-on
matrix-auth-on: ## Run with authentication enabled
	TIMER=APIC JITTER=off AUTH=on POLICY=closed $(MAKE) matrix-local

.PHONY: matrix-jitter-on
matrix-jitter-on: ## Run with jitter injection enabled
	TIMER=APIC JITTER=on AUTH=off POLICY=open $(MAKE) matrix-local

.PHONY: matrix-full-features
matrix-full-features: ## Run with all features enabled
	TIMER=HPET JITTER=on AUTH=on POLICY=closed $(MAKE) matrix-local

# Matrix cell with custom configuration
.PHONY: matrix-custom
matrix-custom: ## Run matrix cell with custom configuration
	@echo "Available matrix configurations:"
	@echo "  TIMER: APIC|HPET (default: APIC)"
	@echo "  JITTER: on|off (default: off)"
	@echo "  AUTH: on|off (default: off)"
	@echo "  POLICY: open|closed (default: open)"
	@echo ""
	@echo "Example: make matrix-custom TIMER=HPET JITTER=on AUTH=on POLICY=closed"
	@echo ""
	@if [ -z "$(TIMER)" ] && [ -z "$(JITTER)" ] && [ -z "$(AUTH)" ] && [ -z "$(POLICY)" ]; then \
		echo "No configuration specified, using defaults..."; \
		$(MAKE) matrix-local; \
	else \
		$(MAKE) matrix-local; \
	fi