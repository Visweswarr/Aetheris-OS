# Local Matrix Runner

## Overview

The Local Matrix Runner enables contributors to reproduce matrix cell configurations locally with the same flags and settings as CI. This provides fast iteration and debugging capabilities without waiting for CI runs, while maintaining consistency with the CI environment.

## Features

- **Matrix Cell Reproduction**: Run any matrix configuration locally
- **CI Consistency**: Same flags and settings as CI runs
- **Clear Results**: PASS/FAIL banners with detailed output
- **Local Artifacts**: Artifacts stored in `.local_artifacts/<config_id>/`
- **Log Management**: Automatic log rotation after 10MB
- **Devcontainer Integration**: VS Code tasks for easy access
- **Makefile Integration**: Simple one-liner commands

## Matrix Configuration

### Environment Variables

The local matrix runner uses the same environment variables as CI:

| Variable | Values | Default | Description |
|----------|--------|---------|-------------|
| `TIMER` | `APIC` \| `HPET` | `APIC` | Timer type (Local APIC or High Precision Event Timer) |
| `JITTER` | `on` \| `off` | `off` | Jitter injection for testing |
| `AUTH` | `on` \| `off` | `off` | Authentication system enabled |
| `POLICY` | `open` \| `closed` | `open` | Capability policy (fail-open vs fail-closed) |

### Configuration Examples

```bash
# APIC timer with default settings
TIMER=APIC JITTER=off AUTH=off POLICY=open

# HPET timer with jitter injection
TIMER=HPET JITTER=on AUTH=off POLICY=open

# Full features enabled
TIMER=HPET JITTER=on AUTH=on POLICY=closed

# Authentication only
TIMER=APIC JITTER=off AUTH=on POLICY=closed
```

## Usage

### Command Line Interface

#### **Basic Usage**
```bash
# Run with default configuration (APIC, no jitter, no auth, open policy)
./tooling/local/run_matrix_cell.sh

# Run specific matrix cell
TIMER=HPET JITTER=on AUTH=on POLICY=closed ./tooling/local/run_matrix_cell.sh

# Run with custom timeout and keep artifacts
./tooling/local/run_matrix_cell.sh --timeout 60 --keep-artifacts

# Clean run with verbose output
./tooling/local/run_matrix_cell.sh --clean --verbose

# Debug mode with custom log level
./tooling/local/run_matrix_cell.sh --debug --log-level debug
```

#### **Command Line Options**
- **`--help`**: Show help message
- **`--verbose`**: Enable verbose output
- **`--debug`**: Enable debug mode with shell tracing
- **`--clean`**: Clean previous artifacts before running
- **`--keep-artifacts`**: Keep artifacts after run (default: cleanup)
- **`--timeout MINUTES`**: Set timeout for tests (default: 30)
- **`--log-level LEVEL`**: Set log level: debug|info|warn|error (default: info)

### Makefile Integration

#### **Preset Targets**
```bash
# APIC timer with default settings
make matrix-apic-default

# HPET timer with default settings
make matrix-hpet-default

# Authentication enabled
make matrix-auth-on

# Jitter injection enabled
make matrix-jitter-on

# All features enabled
make matrix-full-features
```

#### **Custom Configuration**
```bash
# Run with custom configuration
make matrix-local TIMER=HPET JITTER=on AUTH=on POLICY=closed

# Run with custom configuration (alternative)
make matrix-custom TIMER=HPET JITTER=on AUTH=on POLICY=closed

# Interactive custom configuration
make matrix-custom
```

#### **Environment Variable Override**
```bash
# Set environment variables for the entire session
export TIMER=HPET
export JITTER=on
export AUTH=on
export POLICY=closed

# Run matrix cell
make matrix-local
```

## Devcontainer Integration

### VS Code Tasks

The devcontainer provides pre-configured VS Code tasks for running matrix cells:

1. **Run Matrix Cell (APIC Default)**: APIC timer with default settings
2. **Run Matrix Cell (HPET Default)**: HPET timer with default settings
3. **Run Matrix Cell (Auth On)**: Authentication enabled
4. **Run Matrix Cell (Jitter On)**: Jitter injection enabled
5. **Run Matrix Cell (Full Features)**: All features enabled
6. **Run Matrix Cell (Custom)**: Custom configuration
7. **Run Matrix Cell (Local)**: Environment variable configuration

### Accessing Tasks

1. Open Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`)
2. Type "Tasks: Run Task"
3. Select the desired matrix cell task
4. Task will run in the integrated terminal

### Task Configuration

Tasks are configured in `.devcontainer/devcontainer.json` and provide:
- **Clear labeling**: Descriptive task names
- **Shared output**: Results displayed in shared terminal panel
- **Problem matching**: Error detection and reporting
- **Reuse support**: Efficient task execution

## Workflow

### Execution Flow

```
1. Configuration Validation → 2. Artifacts Setup → 3. Kernel Configuration → 4. Build → 5. Tests → 6. Performance → 7. Artifact Collection → 8. Results Display
```

### Step Details

#### **1. Configuration Validation**
- Validates environment variables
- Ensures valid matrix configuration
- Sets default values if not specified

#### **2. Artifacts Setup**
- Creates `.local_artifacts/<config_id>/` directory
- Sets up subdirectories: `logs/`, `tests/`, `kernel/`, `perf/`
- Cleans previous artifacts if `--clean` flag is used

#### **3. Kernel Configuration**
- Generates kernel configuration file
- Sets matrix-specific build flags
- Configures timer, jitter, auth, and policy settings

#### **4. Build**
- Builds kernel with matrix configuration
- Logs build process to `logs/build.log`
- Handles build failures gracefully

#### **5. Tests**
- Runs core tests
- Runs QEMU tests if available
- Logs test results to `logs/tests.log`

#### **6. Performance**
- Runs performance benchmarks
- Collects performance metrics
- Stores results in `perf/` directory

#### **7. Artifact Collection**
- Copies test results, logs, and configuration files
- Creates run summary in Markdown format
- Organizes artifacts by type

#### **8. Results Display**
- Shows clear PASS/FAIL banners
- Displays configuration summary
- Provides artifact location information

## Output and Artifacts

### Console Output

#### **Clear PASS/FAIL Banners**
```
╔══════════════════════════════════════════════════════════════════════════════╗
║                                ✅ ALL TESTS PASSED ✅                          ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

#### **Configuration Summary**
```
📊 Configuration: APIC Timer, Jitter off, Auth off, Policy open
🆔 Config ID: apic_off_off_open
📁 Artifacts: .local_artifacts/apic_off_off_open/
⏱️  Build: ✅ PASS
🧪 Tests: ✅ PASS
📈 Performance: ✅ PASS
```

### Artifact Structure

```
.local_artifacts/
└── apic_off_off_open/
    ├── run_summary.md          # Run summary and results
    ├── logs/                   # Build, test, and performance logs
    │   ├── build.log
    │   ├── tests.log
    │   └── performance.log
    ├── tests/                  # Test results and outputs
    │   ├── test_results.json
    │   └── serial.log
    ├── kernel/                 # Kernel configuration and logs
    │   ├── .config.matrix
    │   └── kernel.log
    └── perf/                   # Performance benchmark results
        ├── metrics.json
        └── benchmark_results.json
```

### Run Summary

The `run_summary.md` file provides a comprehensive overview:

```markdown
# Matrix Cell Run Summary

**Configuration ID**: `apic_off_off_open`
**Configuration Name**: APIC Timer, Jitter off, Auth off, Policy open
**Run Timestamp**: 2024-01-15T10:30:00Z
**Exit Code**: 0

## Matrix Configuration
- **Timer**: APIC
- **Jitter**: off
- **Auth**: off
- **Policy**: open

## Results
- **Build**: ✅ PASS
- **Tests**: ✅ PASS
- **Performance**: ✅ PASS

## Artifacts
- **Logs**: logs/
- **Tests**: tests/
- **Kernel**: kernel/
- **Performance**: perf/

## Next Steps
- All tests passed successfully
```

## Log Management

### Automatic Rotation

Logs are automatically rotated when they exceed 10MB:

- **Rotation Trigger**: 10MB size limit
- **Backup Naming**: `filename.YYYYMMDD_HHMMSS`
- **Compression**: Automatic gzip compression if available
- **Cleanup**: Old logs are preserved for debugging

### Log Files

- **`build.log`**: Kernel build process and output
- **`tests.log`**: Test execution and results
- **`performance.log`**: Performance benchmark execution

### Log Rotation Example

```bash
# Original log file
logs/build.log (15MB)

# After rotation
logs/build.log (0MB) - New log file
logs/build.log.20240115_103000.gz (15MB) - Compressed backup
```

## Error Handling

### Exit Codes

- **`0`**: All tests passed successfully
- **`1`**: Test failures or errors
- **`2`**: Configuration errors
- **`3`**: Build failures

### Failure Scenarios

#### **Configuration Errors**
- Invalid timer value (not APIC or HPET)
- Invalid jitter value (not on or off)
- Invalid auth value (not on or off)
- Invalid policy value (not open or closed)

#### **Build Failures**
- Kernel build errors
- Configuration file issues
- Dependency problems

#### **Test Failures**
- Core test failures
- QEMU test failures
- Performance benchmark failures

### Recovery

- **Graceful Degradation**: Continues processing when possible
- **Clear Error Messages**: Descriptive error information
- **Artifact Preservation**: Failed runs still collect artifacts
- **Logging**: Comprehensive error logging for debugging

## Performance Considerations

### Optimization Features

- **Conditional Execution**: Only runs necessary components
- **Log Rotation**: Prevents disk space issues
- **Artifact Cleanup**: Optional cleanup of old artifacts
- **Timeout Control**: Configurable test timeouts

### Resource Usage

- **Memory**: Efficient memory usage in jobs
- **Disk**: Automatic log rotation and cleanup
- **CPU**: Parallel execution where possible
- **Network**: Local execution, no network overhead

## Troubleshooting

### Common Issues

#### **Permission Errors**
```bash
# Make script executable
chmod +x tooling/local/run_matrix_cell.sh

# Check file permissions
ls -la tooling/local/run_matrix_cell.sh
```

#### **Missing Dependencies**
```bash
# Install required tools
sudo apt-get update
sudo apt-get install -y make gcc qemu-system-x86_64

# Check Rust toolchain
rustup show
```

#### **Build Failures**
```bash
# Check kernel configuration
cat kernel/.config.matrix

# Verify environment variables
env | grep CONFIG_

# Check build logs
tail -f .local_artifacts/*/logs/build.log
```

#### **Test Failures**
```bash
# Check test logs
tail -f .local_artifacts/*/logs/tests.log

# Verify QEMU availability
which qemu-system-x86_64

# Check test results
ls -la .local_artifacts/*/tests/
```

### Debug Mode

Enable debug mode for detailed troubleshooting:

```bash
# Enable debug mode
./tooling/local/run_matrix_cell.sh --debug

# Set custom log level
./tooling/local/run_matrix_cell.sh --debug --log-level debug

# Verbose output
./tooling/local/run_matrix_cell.sh --verbose
```

### Log Analysis

```bash
# View latest logs
tail -f .local_artifacts/*/logs/*.log

# Search for errors
grep -r "ERROR\|FAIL" .local_artifacts/*/logs/

# Check log sizes
du -sh .local_artifacts/*/logs/*.log
```

## Best Practices

### Development Workflow

1. **Start Simple**: Begin with default configurations
2. **Incremental Testing**: Test one feature at a time
3. **Artifact Review**: Always check generated artifacts
4. **Log Monitoring**: Monitor logs during execution
5. **Clean Runs**: Use `--clean` for fresh starts

### Configuration Management

1. **Environment Variables**: Use consistent naming
2. **Default Values**: Rely on sensible defaults
3. **Validation**: Always validate configurations
4. **Documentation**: Document custom configurations

### Performance Optimization

1. **Log Rotation**: Monitor log file sizes
2. **Artifact Cleanup**: Regular cleanup of old artifacts
3. **Timeout Settings**: Adjust timeouts for your environment
4. **Resource Monitoring**: Monitor system resources during runs

## Integration with CI

### Consistency Guarantees

- **Same Flags**: Identical configuration flags as CI
- **Same Environment**: Matching build and test environment
- **Same Output**: Consistent artifact structure
- **Same Validation**: Same test and validation processes

### Local vs CI Differences

| Aspect | Local | CI |
|--------|-------|-----|
| **Execution Time** | Variable | Fixed |
| **Resource Limits** | System dependent | Container limits |
| **Network Access** | Full access | Restricted |
| **Artifact Storage** | Local filesystem | GitHub artifacts |
| **Log Rotation** | 10MB limit | No rotation |

### Migration Path

1. **Local Development**: Use local matrix runner for iteration
2. **CI Validation**: Push changes for CI validation
3. **Result Comparison**: Compare local and CI results
4. **Issue Resolution**: Fix discrepancies locally
5. **Final Validation**: Verify in CI environment

## Future Enhancements

### Planned Features

- **Matrix Visualization**: Web-based matrix status dashboard
- **Result Comparison**: Side-by-side local vs CI comparison
- **Performance Tracking**: Historical performance trend analysis
- **Automated Debugging**: Intelligent error analysis and suggestions

### Integration Improvements

- **IDE Integration**: Native IDE support beyond VS Code
- **CI Integration**: Direct integration with CI systems
- **Cloud Support**: Cloud-based matrix execution
- **Distributed Testing**: Parallel matrix cell execution

## Conclusion

The Local Matrix Runner provides a powerful tool for Polymera OS contributors to iterate quickly on matrix configurations while maintaining consistency with CI. By offering simple one-liner commands, clear output, and comprehensive artifact management, it significantly improves the development experience.

### Key Benefits

- **Fast Iteration**: Local execution without CI wait times
- **CI Consistency**: Same flags and settings as CI runs
- **Clear Results**: Easy-to-understand PASS/FAIL output
- **Comprehensive Artifacts**: Complete test and build information
- **Developer Friendly**: Simple commands and clear documentation

### Getting Started

1. **Basic Usage**: `make matrix-apic-default`
2. **Custom Configuration**: `make matrix-local TIMER=HPET JITTER=on`
3. **Debug Mode**: `./tooling/local/run_matrix_cell.sh --debug`
4. **VS Code Tasks**: Use devcontainer tasks for easy access

The local matrix runner empowers developers to test and debug matrix configurations efficiently, leading to faster development cycles and higher quality code.
