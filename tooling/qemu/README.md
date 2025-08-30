# QEMU Runner Scripts

This directory contains scripts for running the Polymera OS kernel in QEMU for testing and development.

## Scripts

### `run_x86_64.sh` (Linux/macOS/WSL)
Runs the kernel in QEMU with x86_64 emulation.

**Usage:**
```bash
# Run with default kernel location
./tooling/qemu/run_x86_64.sh

# Run with specific kernel image
./tooling/qemu/run_x86_64.sh path/to/kernel.bin

# Run after building with Bazel
bazel build //kernel:kernel_image
./tooling/qemu/run_x86_64.sh bazel-bin/kernel/polymera-kernel.bin
```

### `run_x86_64.bat` (Windows)
Windows version of the QEMU runner.

**Usage:**
```cmd
REM Run with default kernel location
tooling\qemu\run_x86_64.bat

REM Run with specific kernel image
tooling\qemu\run_x86_64.bat path\to\kernel.bin
```

## QEMU Configuration

The scripts use the following QEMU configuration:
- **Architecture**: x86_64
- **Memory**: 512MB
- **Serial**: Redirected to stdio for kernel output
- **Display**: None (headless)
- **Boot**: Direct kernel boot (no bootloader)
- **Flags**: `-no-reboot -no-shutdown` for clean exit

## Output Capture

The bash script automatically captures QEMU output to `serial.log` using `tee`. This allows:
- Real-time viewing of kernel output
- Log file for analysis and CI testing
- Easy debugging of boot issues

## Prerequisites

### Linux/Ubuntu
```bash
sudo apt-get install qemu-system-x86
```

### macOS
```bash
brew install qemu
```

### Windows
Download QEMU from: https://www.qemu.org/download/#windows

## Expected Output

When running a working kernel, you should see output like:
```
Starting QEMU with kernel: bazel-bin/kernel/polymera-kernel.bin
QEMU output will be saved to serial.log
[PolymeraCore] build=0.1.0 target=x86_64-unknown-none
[PolymeraCore] boot::init()
[HAL] x86_64 CPU initialization
[HAL] x86_64 timer initialization
[HAL] enabling x86_64 interrupts
[PHASE1 PASS] boot init sequence completed
```

## Troubleshooting

### "Kernel image not found"
```bash
# Build the kernel first
cd kernel
cargo build --target x86_64-unknown-none --release

# Or with Bazel (when integrated)
bazel build //kernel:kernel_image
```

### "qemu-system-x86_64: command not found"
Install QEMU using the package manager for your platform (see Prerequisites above).

### No output in serial.log
- Check that the kernel binary is valid: `file kernel.bin`
- Verify QEMU is working: `qemu-system-x86_64 --version`
- Try running with longer timeout: `timeout 10s ./run_x86_64.sh`

### Kernel hangs or crashes
- Check for panic messages in the output
- Verify the linker script is correct
- Ensure all dependencies are properly linked

## CI Integration

The scripts are integrated with GitHub Actions in `.github/workflows/phase-1-boot-smoke.yml`:

1. **Build**: Compiles the kernel for x86_64-unknown-none
2. **Test**: Runs QEMU with 3-second timeout
3. **Verify**: Checks for expected boot messages
4. **Artifact**: Uploads serial.log for debugging

This provides automated smoke testing for every kernel change.

## Development Workflow

1. **Make kernel changes**
2. **Build and test locally**:
   ```bash
   bash scripts/test-kernel-build.sh
   ```
3. **Commit and push** - CI will automatically run smoke tests
4. **Debug issues** using uploaded serial.log artifacts

## Advanced Usage

### Custom QEMU Options
Modify the scripts to add additional QEMU options:
```bash
# Add GDB debugging
qemu-system-x86_64 -kernel "$KIMG" -s -S ...

# Increase memory
qemu-system-x86_64 -kernel "$KIMG" -m 1024 ...

# Add disk image
qemu-system-x86_64 -kernel "$KIMG" -drive file=disk.img ...
```

### Longer Testing
For longer stability testing:
```bash
# Run for 30 seconds
timeout 30s ./tooling/qemu/run_x86_64.sh

# Run until manual stop (Ctrl+C)
./tooling/qemu/run_x86_64.sh
```