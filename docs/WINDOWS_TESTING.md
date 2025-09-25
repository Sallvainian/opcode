# Windows Testing & Validation Framework

This document describes the comprehensive Windows testing suite for the Opcode project, designed to ensure production-readiness on Windows platforms.

## Overview

The Windows testing framework provides comprehensive validation across multiple dimensions:

- **Performance Benchmarking** - Memory, startup time, CPU usage, file operations
- **Build Validation** - Debug/release builds, dependency validation, feature testing
- **Installer Testing** - MSI and NSIS installer generation and validation
- **Security & Compatibility** - UAC, permissions, antivirus compatibility, Windows 10/11 support
- **Integration Testing** - Windows-specific API and system integration

## Test Modules

### 1. Performance Testing (`windows_performance.rs`)

**Purpose**: Validate performance requirements and benchmarks

**Key Tests**:
- `benchmark_startup_time()` - Application startup <2 seconds
- `benchmark_memory_usage()` - Memory usage <50MB at idle
- `benchmark_file_operations()` - File I/O performance targets
- `benchmark_cpu_usage_monitoring()` - CPU usage measurement capabilities
- `benchmark_network_latency()` - Network stack performance
- `benchmark_disk_io_performance()` - Large file I/O performance
- `benchmark_windows_api_performance()` - Windows API call latency

**Performance Targets**:
- Startup time: <2 seconds
- Memory usage: <50MB at idle
- File operations: 100 files <500ms create, <100ms read, <200ms delete
- API calls: <100μs average

### 2. Build Validation (`windows_build_validation.rs`)

**Purpose**: Ensure reliable compilation and build processes

**Key Tests**:
- `test_cargo_build_debug()` - Debug build compilation
- `test_cargo_build_release()` - Release build with optimizations
- `test_dependency_validation()` - All dependencies resolve correctly
- `test_windows_specific_features()` - Windows-specific code compilation
- `test_feature_flags()` - Feature flag combinations
- `test_build_reproducibility()` - Consistent build outputs
- `test_clippy_linting()` - Code quality checks
- `test_documentation_build()` - Documentation generation

**Validation Criteria**:
- Both debug and release builds succeed
- Release binary smaller than debug (optimizations working)
- All dependencies in tree
- No compilation errors
- Clean clippy results

### 3. Installer Testing (`windows_installer_validation.rs`)

**Purpose**: Validate Windows installer generation and integrity

**Key Tests**:
- `test_msi_installer_generation()` - WiX MSI installer creation
- `test_nsis_installer_generation()` - NSIS installer creation
- `test_msi_installation_simulation()` - MSI installation testing
- `test_installer_metadata()` - Configuration validation
- `test_installer_icons()` - Icon file validation
- `test_updater_artifact_generation()` - Updater package creation

**Installer Validation**:
- MSI files generated with correct size and structure
- NSIS executables created (if configured)
- Icons present and valid
- Metadata properly configured
- Updater signatures generated (if signing enabled)

**Note**: Installer tests require `--ignored` flag and full build:
```bash
cargo test windows_installer_validation --ignored
```

### 4. Security & Compatibility (`windows_security_compatibility.rs`)

**Purpose**: Validate security features and Windows version compatibility

**Security Tests**:
- `test_uac_elevation_detection()` - UAC and admin privilege detection
- `test_windows_security_features()` - Windows Defender and Firewall status
- `test_file_system_permissions()` - Proper permission boundaries
- `test_antivirus_compatibility()` - Antivirus software compatibility
- `test_windows_security_boundaries()` - System file access restrictions

**Compatibility Tests**:
- `test_windows_version_compatibility()` - Windows 10/11 detection
- `test_windows_feature_availability()` - Required Windows features
- `test_windows_api_compatibility()` - Windows API access
- `test_windows_service_integration()` - Windows services interaction
- `test_windows_registry_security()` - Registry access patterns

**Compatibility Matrix**:
- Windows 10 (version 1903+): ✅ Supported
- Windows 11: ✅ Supported
- Required features: PowerShell, WMI, Registry, Task Management

### 5. Integration Testing (`windows_integration.rs`)

**Purpose**: Extended Windows-specific functionality testing

**Enhanced Coverage** (added to existing tests):
- Process management with elevation detection
- Advanced registry operations
- Network configuration testing
- Windows service enumeration
- Power management integration
- Extended file system testing
- UNC path and long path support

### 6. Test Runner & Coverage (`windows_test_runner.rs`)

**Purpose**: Orchestrate comprehensive test execution and reporting

**Key Features**:
- `run_comprehensive_windows_test_suite()` - Full test suite execution
- `analyze_test_coverage()` - Coverage analysis and reporting
- `validate_test_environment()` - Environment prerequisites check
- `benchmark_test_execution_time()` - Test performance monitoring

## Running Tests

### PowerShell Test Suite Runner

The primary way to run comprehensive Windows tests:

```powershell
# Run full test suite
.\scripts\windows-test-suite.ps1 -Full

# Run specific test categories
.\scripts\windows-test-suite.ps1 -Performance
.\scripts\windows-test-suite.ps1 -Security
.\scripts\windows-test-suite.ps1 -Build
.\scripts\windows-test-suite.ps1 -Installer

# Generate coverage report
.\scripts\windows-test-suite.ps1 -Coverage -Verbose
```

### Manual Test Execution

```bash
# Run all Windows tests
cargo test --target x86_64-pc-windows-msvc

# Run specific test modules
cargo test windows_performance --target x86_64-pc-windows-msvc
cargo test windows_build_validation --target x86_64-pc-windows-msvc
cargo test windows_security_compatibility --target x86_64-pc-windows-msvc

# Run installer tests (requires full build)
cargo test windows_installer_validation --target x86_64-pc-windows-msvc -- --ignored

# Run with verbose output
cargo test --target x86_64-pc-windows-msvc -- --nocapture
```

## GitHub Actions Integration

The Windows build workflow (`.github/workflows/build-windows.yml`) includes:

- Automated test execution on Windows runners
- MSI and NSIS installer builds
- Test result reporting
- Artifact collection for installers and updater packages

## Test Reports

Tests generate comprehensive reports in the `test-reports/` directory:

- `windows-test-report.md` - Executive summary and detailed results
- Individual test output files for debugging
- Performance benchmarking data
- Coverage analysis results

## Prerequisites

### Required Tools
- Rust with `x86_64-pc-windows-msvc` target
- PowerShell 5.0+ for advanced tests
- Windows SDK (for some Windows API tests)
- Git for repository operations

### Optional Tools (for full coverage)
- WiX Toolset (for MSI installer testing)
- NSIS (for NSIS installer testing)
- Code signing certificate (for updater testing)

### Environment Setup

```bash
# Install Windows MSVC target
rustup target add x86_64-pc-windows-msvc

# Verify installation
rustup target list --installed | grep x86_64-pc-windows-msvc
```

## Performance Standards

The test suite enforces these performance standards for production readiness:

| Metric | Target | Test Method |
|--------|---------|------------|
| Startup Time | <2 seconds | Process execution timing |
| Memory Usage (Idle) | <50MB | Windows Task Manager API |
| File Operations (100 files) | Create <500ms, Read <100ms, Delete <200ms | Filesystem benchmarking |
| Windows API Calls | <100μs average | High-frequency API timing |
| Network Stack | Localhost ping <5ms | Network latency testing |

## Security Validation

The security testing validates:

- **UAC Integration**: Proper elevation detection and handling
- **Permission Boundaries**: Cannot access restricted system files
- **Registry Security**: Safe registry key access patterns
- **Antivirus Compatibility**: No false positives with common AV software
- **Windows Security Features**: Integration with Windows Defender/Firewall

## Troubleshooting

### Common Issues

1. **Build Failures**:
   - Ensure `x86_64-pc-windows-msvc` target is installed
   - Check Windows SDK installation
   - Verify all dependencies in `Cargo.toml`

2. **Test Permission Errors**:
   - Some tests require administrator privileges
   - Run PowerShell as Administrator for full test suite
   - UAC may block certain operations

3. **Installer Test Failures**:
   - Installer tests require full `cargo tauri build`
   - WiX Toolset must be installed for MSI tests
   - NSIS must be installed for NSIS tests

4. **Performance Test Variations**:
   - Performance may vary by hardware
   - Tests include reasonable tolerances
   - Virtual machines may have different performance characteristics

### Debug Mode

Enable verbose output for debugging:

```powershell
.\scripts\windows-test-suite.ps1 -Verbose
```

Or with cargo:

```bash
cargo test --target x86_64-pc-windows-msvc -- --nocapture
```

## Continuous Integration

The Windows testing framework integrates with CI/CD:

1. **Pre-commit**: Runs fast subset of tests
2. **Pull Request**: Full test suite execution
3. **Release**: Complete validation including installer tests
4. **Nightly**: Extended compatibility and performance testing

## Contributing

When adding new Windows-specific features:

1. Add corresponding tests to appropriate test modules
2. Update performance benchmarks if needed
3. Ensure security testing covers new attack surfaces
4. Update this documentation with new test capabilities

## Future Enhancements

Planned improvements to the testing framework:

- **Hardware-specific testing**: Different CPU architectures
- **Windows version matrix**: Automated testing across multiple Windows versions
- **Extended antivirus testing**: Integration with multiple AV vendors
- **Performance regression detection**: Historical performance tracking
- **Accessibility testing**: Windows accessibility API compliance