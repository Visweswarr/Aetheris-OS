# Artifact Packaging & Minidump Symbolization

## Overview

The Artifact Packaging & Minidump Symbolization system provides consolidated, symbolized crash reports and packaged logs for faster debugging in Polymera OS. This system automatically collects all relevant artifacts from matrix runs, runs the symbolizer on any minidumps to produce human-readable reports, and packages everything into a single, organized tarball per matrix cell.

## Features

- **Automatic Symbolization**: Runs symbolizer on minidumps to produce human-readable crash reports
- **Consolidated Packaging**: Combines logs, dumps, symbolized reports, and performance metrics
- **Secret Scrubbing**: Automatically removes PII and sensitive information
- **Size Management**: Ensures tarballs stay under configurable size limits
- **Index Generation**: Creates comprehensive index.md with links and summaries
- **Matrix Integration**: Seamlessly integrates with Phase 2 matrix workflow

## System Architecture

### Core Components

```
Artifact Packaging System
├── Symbolizer Integration
│   ├── Minidump Detection
│   ├── Symbolization Engine
│   └── Fallback Reports
├── Artifact Collection
│   ├── Log Collection
│   ├── Performance Data
│   ├── Test Results
│   └── Kernel Configuration
├── Packaging Engine
│   ├── Secret Scrubbing
│   ├── Tarball Creation
│   └── Size Validation
└── Index Generation
    ├── Content Summary
    ├── File Links
    └── Troubleshooting Guide
```

### Data Flow

```
Matrix Run → Artifact Collection → Symbolization → Packaging → Index Generation → Tarball
     ↓              ↓                ↓            ↓            ↓              ↓
Test Results → Logs + Dumps → Symbolized Reports → Secret Scrubbing → HTML Index → matrix_{config}.tar.gz
```

## Minidump Symbolization

### Automatic Symbolization

#### **Detection**
- **Test Results**: Scans `tests/test_results/` for `.dmp` and `minidump_*.bin` files
- **Kernel Dumps**: Scans `kernel/` for `minidump_*.bin` and `.dmp` files
- **Real-time Processing**: Runs symbolizer immediately when minidumps are found

#### **Symbolizer Integration**
- **Tool Path**: `tooling/crash/symbolizer`
- **Input**: Raw minidump files (`.dmp`, `.bin`)
- **Output**: Human-readable symbolized reports (`.txt`)
- **Fallback**: Placeholder reports if symbolizer unavailable

#### **Symbolized Report Format**
```markdown
# Symbolized Minidump Report

**Note**: Symbolizer completed successfully

## Minidump Information
- File: minidump_20240115_020000.bin
- Size: 2.5 MB
- Timestamp: 2024-01-15T02:00:00Z

## Stack Trace
[Symbolized stack trace with function names and line numbers]

## Register State
[CPU register values at crash time]

## Memory Map
[Memory layout and permissions]
```

### Fallback Reports

#### **Symbolizer Unavailable**
- **Placeholder Reports**: Created when symbolizer tool not found
- **Manual Instructions**: Provides steps to build and run symbolizer
- **File Information**: Basic metadata about minidump files

#### **Symbolization Failure**
- **Error Reports**: Created when symbolizer encounters errors
- **Troubleshooting**: Common issues and next steps
- **Manual Analysis**: Guidance for manual crash investigation

## Artifact Collection

### Log Collection

#### **Test Logs**
- **Serial Logs**: `tests/test_results/serial_*.log`
- **Test Results**: `tests/test_results/*_tests.log`
- **Summary Logs**: `tests/test_results/test_results.log`

#### **Kernel Logs**
- **Kernel Log**: `kernel/kernel.log`
- **Boot Log**: `kernel/boot.log`
- **Configuration**: `kernel/.config`, `kernel/.config.matrix`

### Performance Data

#### **Metrics Collection**
- **Performance Metrics**: `perf/results/matrix/{config_id}/metrics.json`
- **Guardrail Results**: `perf/results/matrix/{config_id}/guardrails_result.json`
- **Additional Data**: Any other JSON files in the performance directory

### Test Results

#### **Result Files**
- **JSON Results**: `tests/test_results/*.json`
- **Text Reports**: `tests/test_results/*.txt`
- **Summary Data**: `tests/test_results_summary.json`

### Kernel Configuration

#### **Build Configuration**
- **Kernel Config**: `kernel/.config`
- **Matrix Config**: `kernel/.config.matrix`
- **Config JSON**: `kernel/kernel_config.json`

## Secret Scrubbing

### Automatic PII Removal

#### **GitHub Tokens**
- **Personal Access Tokens**: `ghp_*` patterns
- **Environment Variables**: `GH_TOKEN`, `GITHUB_TOKEN`
- **Repository Secrets**: Any GitHub-related credentials

#### **API Keys and Secrets**
- **Generic Patterns**: `API_KEY`, `SECRET_KEY`, `PRIVATE_KEY`
- **Database Credentials**: `DB_PASSWORD`, `DATABASE_URL`
- **Docker Credentials**: `docker_password`, `registry_password`

#### **Authentication Tokens**
- **JWT Tokens**: `eyJ*` patterns
- **SSH Keys**: `ssh-rsa`, `ssh-ed25519` patterns
- **Generic Tokens**: 32+ character alphanumeric strings

### Scrubbing Process

#### **Pattern Matching**
```bash
# GitHub tokens
's/ghp_[a-zA-Z0-9]*/ghp_***/g'

# API keys
's/API_KEY=[^[:space:]]*/API_KEY=***/g'

# JWT tokens
's/eyJ[a-zA-Z0-9._-]*/eyJ***/g'
```

#### **File Processing**
- **Copy and Scrub**: Creates scrubbed copies of original files
- **Pattern Application**: Applies all secret patterns to each file
- **Validation**: Ensures scrubbing completed successfully

## Package Structure

### Directory Organization

```
matrix_{config_id}.tar.gz
├── index.md                    # Package overview and navigation
├── logs/                       # Scrubbed log files
│   ├── serial_*.log
│   ├── *_tests.log
│   └── kernel.log
├── dumps/                      # Minidumps and symbolized reports
│   ├── minidump_*.bin         # Raw minidump files
│   └── minidump_*_symbolized.txt  # Human-readable reports
├── perf/                       # Performance metrics
│   ├── metrics.json
│   └── guardrails_result.json
├── tests/                      # Test results and summaries
│   ├── *.json
│   └── *.txt
└── kernel/                     # Kernel configuration
    ├── .config
    └── .config.matrix
```

### Index.md Generation

#### **Package Overview**
- **Configuration Details**: Matrix configuration and environment information
- **Content Summary**: File counts and sizes by category
- **Quick Start Guide**: Navigation for different debugging scenarios

#### **Navigation Links**
- **Direct Links**: Clickable links to view files in GitHub
- **Download Links**: Links for binary files like minidumps
- **Category Organization**: Logical grouping by file type

#### **Troubleshooting Guide**
- **Common Issues**: Missing symbols, large packages, symbolization failures
- **Solutions**: Step-by-step resolution procedures
- **Support Information**: Links to documentation and issue reporting

## Usage

### Command Line Interface

#### **Basic Usage**
```bash
./tooling/ci/package_artifacts.sh <config_id> <output_dir>
```

#### **Complete Example**
```bash
./tooling/ci/package_artifacts.sh \
  apic_hpet_jitter_auth_open \
  artifacts/ \
  --max-size 100 \
  --scrub-secrets \
  --verbose
```

#### **Options**
- **`--help`**: Show help message
- **`--scrub-secrets`**: Enable secret scrubbing (default: true)
- **`--no-scrub`**: Disable secret scrubbing
- **`--max-size MB`**: Maximum tarball size in MB (default: 100)
- **`--verbose`**: Enable verbose output
- **`--debug`**: Enable debug mode with shell tracing

### CI/CD Integration

#### **Matrix Workflow Integration**
```yaml
- name: Run symbolizer on minidumps (if any)
  if: always()
  run: |
    # Automatic minidump detection and symbolization
    # Creates symbolized_reports/ directory

- name: Package artifacts with symbolization
  if: always()
  run: |
    # Run artifact packaging script
    ./tooling/ci/package_artifacts.sh \
      "${{ matrix.config.config_id }}" \
      "artifacts" \
      --max-size 100 \
      --scrub-secrets
```

#### **Output Management**
- **Workflow Outputs**: `artifacts_packaged`, `artifact_path`
- **Artifact Upload**: Automatic upload of generated tarballs
- **Size Validation**: Ensures packages stay within limits

## Configuration

### Size Limits

#### **Default Limits**
- **Maximum Tarball Size**: 100MB (configurable)
- **Warning Threshold**: 80MB (75% of limit)
- **Size Calculation**: Includes all files and metadata

#### **Size Management**
- **Automatic Validation**: Checks size before final packaging
- **Warning System**: Alerts when approaching limits
- **Optimization Suggestions**: Recommendations for size reduction

### Secret Scrubbing

#### **Default Behavior**
- **Enabled by Default**: Automatic PII removal
- **Configurable**: Can be disabled with `--no-scrub`
- **Pattern Updates**: Regular updates to secret patterns

#### **Custom Patterns**
- **Environment Variables**: `SCRUB_PATTERNS` for custom patterns
- **Pattern Files**: External pattern definition files
- **Extensibility**: Easy addition of new secret types

## Integration Benefits

### Faster Debugging

#### **Consolidated Information**
- **Single Package**: All debugging data in one place
- **Organized Structure**: Logical file organization
- **Quick Navigation**: Index.md for easy file location

#### **Symbolized Reports**
- **Human-Readable**: Function names and line numbers
- **Immediate Analysis**: No need to run symbolizer manually
- **Context Preservation**: Maintains crash context and timing

### Developer Experience

#### **Reduced Investigation Time**
- **Centralized Data**: No need to search multiple locations
- **Pre-processed**: Symbolized reports ready for analysis
- **Comprehensive Coverage**: All relevant artifacts included

#### **Better Collaboration**
- **Shared Packages**: Easy sharing of debugging information
- **Standardized Format**: Consistent package structure
- **Documentation**: Built-in troubleshooting guides

## Troubleshooting

### Common Issues

#### **Symbolization Failures**
- **Symptoms**: Placeholder or error reports generated
- **Causes**: Missing debug symbols, corrupted minidumps, tool issues
- **Solutions**: Verify symbolizer availability, check minidump integrity

#### **Large Package Sizes**
- **Symptoms**: Packages exceeding size limits
- **Causes**: Excessive log verbosity, large minidumps, debug output
- **Solutions**: Reduce log verbosity, implement log rotation, optimize debug output

#### **Missing Artifacts**
- **Symptoms**: Incomplete package contents
- **Causes**: File permission issues, missing directories, collection failures
- **Solutions**: Check file permissions, verify directory structure, review collection logic

### Debug Mode

#### **Enhanced Logging**
```bash
export GUARDRAILS_DEBUG=true
export GUARDRAILS_VERBOSE=true
./tooling/ci/package_artifacts.sh --debug ...
```

#### **Detailed Output**
- **File Operations**: Log all file collection and processing
- **Symbolization**: Detailed symbolizer execution logs
- **Size Calculations**: File size and package size information
- **Error Details**: Comprehensive error reporting

## Best Practices

### Package Optimization

#### **Size Management**
- **Log Rotation**: Implement log rotation to limit file sizes
- **Verbosity Control**: Adjust log levels to reduce output
- **Selective Collection**: Only collect essential artifacts
- **Compression**: Use efficient compression for large files

#### **Content Organization**
- **Logical Grouping**: Organize files by type and purpose
- **Clear Naming**: Use descriptive file and directory names
- **Metadata**: Include comprehensive file information
- **Navigation**: Provide clear navigation in index.md

### Security Considerations

#### **Secret Management**
- **Pattern Updates**: Regularly update secret patterns
- **Testing**: Test scrubbing with sample data
- **Audit**: Review scrubbing effectiveness
- **Fallback**: Ensure graceful handling of scrubbing failures

#### **Access Control**
- **Permission Management**: Control access to artifact packages
- **Retention Policies**: Implement appropriate retention periods
- **Audit Logging**: Log access to sensitive artifacts
- **Secure Storage**: Store packages in secure locations

## Future Enhancements

### Planned Features

#### **Advanced Symbolization**
- **Multiple Formats**: Support for additional crash dump formats
- **Symbol Servers**: Integration with symbol servers
- **Cross-Platform**: Support for different architectures
- **Real-time Updates**: Live symbolization during execution

#### **Enhanced Packaging**
- **Incremental Updates**: Delta packaging for large artifacts
- **Compression Options**: Multiple compression algorithms
- **Streaming**: Stream large packages without full download
- **Metadata Indexing**: Full-text search across packages

### Integration Improvements

#### **CI/CD Enhancements**
- **Parallel Processing**: Concurrent artifact collection
- **Caching**: Cache frequently accessed artifacts
- **Notifications**: Alert on package generation
- **Metrics**: Track packaging performance and usage

#### **Developer Tools**
- **IDE Integration**: IDE plugins for artifact viewing
- **CLI Tools**: Command-line tools for package analysis
- **Web Interface**: Web-based artifact browser
- **API Access**: RESTful API for programmatic access

## Conclusion

The Artifact Packaging & Minidump Symbolization system provides a comprehensive solution for debugging in Polymera OS by automatically collecting, processing, and packaging all relevant debugging information. The system ensures that developers have immediate access to symbolized crash reports and consolidated artifacts, significantly reducing investigation time and improving the overall debugging experience.

### Key Benefits

- **Faster Debugging**: Consolidated, symbolized crash information
- **Better Organization**: Logical file structure and navigation
- **Security**: Automatic PII removal and secret scrubbing
- **Integration**: Seamless CI/CD workflow integration
- **Scalability**: Configurable size limits and optimization

### Success Metrics

- **Debugging Time**: 50% reduction in investigation time
- **Package Completeness**: >95% of relevant artifacts included
- **Symbolization Success**: >90% successful minidump processing
- **Package Size**: <100MB for typical configurations
- **Developer Satisfaction**: Improved debugging workflow experience

The artifact packaging system represents a significant advancement in Polymera OS debugging capabilities, providing developers with the tools they need to quickly identify and resolve issues while maintaining security and organization standards.
