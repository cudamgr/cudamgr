# CudaMgr Developer Guide

This document contains technical details, architecture information, and development guidelines for CudaMgr.

## 🏗️ Architecture Overview

CudaMgr is built using a modular architecture with clear separation of concerns:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   CLI Layer     │    │  System Layer   │    │ Install Layer   │
│                 │    │                 │    │                 │
│ • Commands      │    │ • GPU Detection │    │ • Downloaders   │
│ • Validation    │    │ • Driver Check  │    │ • Installers    │
│ • Output        │    │ • Compatibility │    │ • Validators    │
│ • Interactive   │    │ • Security      │    │ • Cleanup       │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
         ┌─────────────────┐    ┌─────────────────┐
         │ Version Layer   │    │ Config Layer    │
         │                 │    │                 │
         │ • Registry      │    │ • Environment   │
         │ • Switcher      │    │ • PATH Mgmt     │
         │ • Resolver      │    │ • Symlinks      │
         │ • Validation    │    │ • Shell Config  │
         └─────────────────┘    └─────────────────┘
```

## 📁 Project Structure

```
src/
├── main.rs              # Application entry point
├── lib.rs               # Library root and public API
├── error.rs             # Centralized error handling
│
├── cli/                 # Command-line interface layer
│   ├── mod.rs           # CLI module exports
│   ├── commands.rs      # Command definitions and handlers
│   ├── output.rs        # Formatted output utilities
│   ├── interactive.rs   # User interaction prompts
│   └── tests.rs         # CLI unit tests
│
├── system/              # System detection and compatibility
│   ├── mod.rs           # System module exports
│   ├── gpu.rs           # GPU detection and info
│   ├── driver.rs        # Driver version detection
│   ├── compiler.rs      # Compiler detection (gcc, nvcc)
│   ├── distro.rs        # OS/distribution detection
│   ├── storage.rs       # Disk space and permissions
│   ├── security.rs      # Security validation
│   ├── cuda.rs          # CUDA-specific utilities
│   ├── tests.rs         # System unit tests
│   └── platform_tests.rs # Platform-specific tests
│
├── install/             # Installation management
│   ├── mod.rs           # Install module exports
│   ├── downloader.rs    # Package downloading logic
│   ├── installer.rs     # Platform-specific installation
│   ├── validator.rs     # Installation validation
│   └── cleanup.rs       # Cleanup and rollback utilities
│
├── version/             # Version management
│   ├── mod.rs           # Version module exports
│   ├── registry.rs      # Version registry and metadata
│   ├── switcher.rs      # Version switching logic
│   └── resolver.rs      # Version resolution and conflicts
│
└── config/              # Configuration management
    ├── mod.rs           # Config module exports
    ├── env.rs           # Environment variable management
    ├── path.rs          # PATH manipulation
    ├── symlink.rs       # Symlink management
    ├── shell.rs         # Shell configuration
    └── tests.rs         # Config unit tests

tests/                   # Integration tests
├── cli_integration_test.rs  # CLI integration tests
└── ...                      # Additional integration tests

.kiro/                   # Kiro AI specification files
├── specs/
│   └── cudamgr/
│       ├── requirements.md  # Feature requirements
│       ├── design.md        # System design document
│       └── tasks.md         # Implementation task list
```

## 🔧 Dependencies

### Core Dependencies

| Crate | Version | Purpose | Features |
|-------|---------|---------|----------|
| `clap` | 4.4 | CLI argument parsing | `derive` for proc macros |
| `tokio` | 1.0 | Async runtime | `full` for all features |
| `serde` | 1.0 | Serialization | `derive` for proc macros |
| `thiserror` | 1.0 | Error handling | Custom error types |
| `reqwest` | 0.11 | HTTP client | `json` for JSON support |
| `tracing` | 0.1 | Structured logging | Core logging framework |
| `tracing-subscriber` | 0.3 | Log subscriber | `env-filter` for filtering |
| `chrono` | 0.4 | Date/time handling | `serde` for serialization |
| `dirs` | 5.0 | Platform directories | Cross-platform paths |
| `serde_json` | 1.0 | JSON serialization | JSON format support |
| `async-trait` | 0.1 | Async traits | Enable async in traits |

### Development Dependencies

```toml
[dev-dependencies]
tempfile = "3.0"      # Temporary files for testing
mockall = "0.11"      # Mocking framework
criterion = "0.5"     # Benchmarking
proptest = "1.0"      # Property-based testing
```

## 🧪 Testing Strategy

### Test Categories

1. **Unit Tests** (`src/**/*tests.rs`)
   - Test individual functions and modules
   - Mock external dependencies
   - Fast execution, no I/O

2. **Integration Tests** (`tests/*.rs`)
   - Test CLI interface end-to-end
   - Test module interactions
   - Real command execution

3. **Property Tests** (planned)
   - Test invariants with random inputs
   - Version parsing edge cases
   - Path manipulation safety

### Running Tests

```bash
# All tests
cargo test

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*'

# Specific module tests
cargo test --lib cli
cargo test --lib system
cargo test --lib config

# Test with output
cargo test -- --nocapture

# Test with logging
RUST_LOG=debug cargo test

# Specific test
cargo test test_install_args_validation

# Test coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Test Patterns

#### CLI Testing Pattern
```rust
#[test]
fn test_command_validation() {
    let args = InstallArgs {
        version: "11.8".to_string(),
        force: false,
        skip_driver: false,
    };
    assert!(args.validate().is_ok());
    
    let invalid_args = InstallArgs {
        version: "".to_string(),
        force: false,
        skip_driver: false,
    };
    assert!(invalid_args.validate().is_err());
}
```

#### Async Handler Testing Pattern
```rust
#[tokio::test]
async fn test_command_execution() {
    let handler = DoctorHandler::new(DoctorArgs { verbose: true });
    let result = handler.execute().await;
    
    // Should return "not implemented" error for now
    assert!(result.is_err());
    match result {
        Err(CudaMgrError::Cli(msg)) => {
            assert!(msg.contains("not yet implemented"));
        }
        _ => panic!("Expected CLI error"),
    }
}
```

### Manual Testing Checklist

#### CLI Help System
```bash
# Main help
cargo run -- --help                    # ✅ Should show main help
cargo run -- --version                 # ✅ Should show version

# Command help
cargo run -- doctor --help             # ✅ Should show doctor help
cargo run -- install --help            # ✅ Should show install help
cargo run -- use --help                # ✅ Should show use help
cargo run -- list --help               # ✅ Should show list help
cargo run -- uninstall --help          # ✅ Should show uninstall help
cargo run -- logs --help               # ✅ Should show logs help
```

#### Command Validation
```bash
# Valid commands (should show "not implemented")
cargo run -- doctor                    # ✅ Exit code 1, "not implemented"
cargo run -- doctor --verbose          # ✅ Exit code 1, "not implemented"
cargo run -- install 11.8              # ✅ Exit code 1, "not implemented"
cargo run -- install 12.0 --force      # ✅ Exit code 1, "not implemented"
cargo run -- use 11.8 --install        # ✅ Exit code 1, "not implemented"
cargo run -- list --available          # ✅ Exit code 1, "not implemented"
cargo run -- uninstall 11.8 --yes      # ✅ Exit code 1, "not implemented"
cargo run -- logs --lines 100          # ✅ Exit code 1, "not implemented"

# Invalid commands (should show validation errors)
cargo run -- install                   # ❌ Missing version argument
cargo run -- install ""                # ❌ Empty version
cargo run -- install "bad-version!"    # ❌ Invalid version format
cargo run -- use                       # ❌ Missing version argument
cargo run -- use ""                    # ❌ Empty version
cargo run -- uninstall                 # ❌ Missing version argument
cargo run -- logs --lines 0            # ❌ Invalid line count
cargo run -- logs --lines 20000        # ❌ Line count too high

# Unknown commands/flags
cargo run -- invalid-command           # ❌ Unknown command
cargo run -- doctor --invalid-flag     # ❌ Unknown flag
```

## 🏗️ Development Workflow

### Building

```bash
# Debug build (fast compilation, includes debug info)
cargo build

# Release build (optimized, slower compilation)
cargo build --release

# Check without building (fastest)
cargo check

# Check all targets and features
cargo check --all-targets --all-features
```

### Code Quality

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Lint code
cargo clippy

# Lint with all targets
cargo clippy --all-targets --all-features

# Lint with warnings as errors
cargo clippy -- -D warnings

# Security audit
cargo audit

# Check for unused dependencies
cargo machete
```

### Debugging

```bash
# Debug logging
RUST_LOG=debug cargo run -- doctor --verbose

# Trace logging (very verbose)
RUST_LOG=trace cargo run -- install 11.8

# Module-specific logging
RUST_LOG=cudamgr::cli=debug cargo run -- --help
RUST_LOG=cudamgr::system=trace cargo run -- doctor

# Backtrace on panic
RUST_BACKTRACE=1 cargo run -- doctor

# Full backtrace
RUST_BACKTRACE=full cargo run -- doctor
```

## 🎯 Implementation Status

### Phase 1: CLI Framework ✅ **COMPLETED**

**Task 3: Build CLI framework and command structure**
- ✅ CLI command definitions using clap (doctor, install, use, list, uninstall, logs)
- ✅ Command handler trait and basic routing logic
- ✅ Formatted output utilities with progress indicators
- ✅ Interactive prompts and confirmation dialogs
- ✅ Unit tests for CLI parsing and command routing

**Implemented Components:**
- `CommandHandler` trait with async support
- Individual handler structs for each command
- `CommandRouter` for dispatching commands
- Argument validation for all commands
- `OutputFormatter` with progress bars and spinners
- `Interactive` module with various prompt types
- Comprehensive test coverage

### Phase 2: System Detection 🚧 **IN PROGRESS**

**Upcoming Tasks:**
- GPU detection and compatibility checking
- Driver version detection and validation
- OS/distribution detection
- Package manager identification
- Storage space and permission validation
- Security checks and requirements

### Phase 3: Installation Management ⏳ **PENDING**

**Planned Tasks:**
- Package downloading with progress tracking
- Platform-specific installation logic
- Installation validation and verification
- Cleanup and rollback capabilities
- Dependency management

### Phase 4: Version Management ⏳ **PENDING**

**Planned Tasks:**
- Version registry and metadata management
- Version switching and environment setup
- Configuration and PATH management
- Shell integration and activation

## 🔍 Code Patterns

### Error Handling Pattern

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CudaMgrError {
    #[error("System check failed: {0}")]
    System(#[from] SystemError),
    
    #[error("CLI error: {0}")]
    Cli(String),
}

pub type CudaMgrResult<T> = Result<T, CudaMgrError>;
```

### Command Handler Pattern

```rust
#[async_trait]
pub trait CommandHandler {
    async fn execute(&self) -> CudaMgrResult<()>;
    fn validate(&self) -> CudaMgrResult<()> {
        Ok(())
    }
}

pub struct InstallHandler {
    args: InstallArgs,
}

#[async_trait]
impl CommandHandler for InstallHandler {
    async fn execute(&self) -> CudaMgrResult<()> {
        // Implementation here
        Ok(())
    }
}
```

### Output Formatting Pattern

```rust
// Success message
OutputFormatter::success("Installation completed successfully");

// Progress tracking
let mut progress = ProgressBar::new(100, "Downloading CUDA 11.8".to_string());
progress.update(50);
progress.finish();

// Interactive confirmation
if Interactive::confirm("Install CUDA 11.8?")? {
    // Proceed with installation
}
```

## 📊 Performance Considerations

### Async Design
- All I/O operations are async to prevent blocking
- Command handlers use `async-trait` for consistency
- Tokio runtime handles concurrency

### Memory Management
- Streaming downloads to avoid loading large files in memory
- Progress tracking with minimal overhead
- Efficient string handling for version parsing

### Error Propagation
- `thiserror` for zero-cost error handling
- Early validation to fail fast
- Structured error types for better debugging

## 🔒 Security Considerations

### Input Validation
- All user inputs are validated before processing
- Version strings are sanitized and checked
- Path traversal protection for file operations

### Download Security
- HTTPS-only downloads
- Checksum verification for all packages
- Signature validation (planned)

### Installation Security
- Permission checks before installation
- Temporary file cleanup
- Rollback capabilities for failed installations

## 📈 Future Enhancements

### Planned Features
- GUI interface using `tauri` or `egui`
- Plugin system for custom CUDA variants
- Docker integration for containerized environments
- CI/CD integration helpers
- Configuration profiles for different projects

### Performance Optimizations
- Parallel downloads for multiple components
- Incremental updates and delta patches
- Local caching and mirror support
- Background version checking

## 🤝 Contributing Guidelines

### Code Style
- Follow Rust standard formatting (`cargo fmt`)
- Use `clippy` recommendations
- Write comprehensive tests for new features
- Document public APIs with rustdoc

### Pull Request Process
1. Create feature branch from `main`
2. Implement changes with tests
3. Run full test suite: `cargo test`
4. Check code quality: `cargo clippy`
5. Update documentation if needed
6. Submit PR with clear description

### Testing Requirements
- All new code must have unit tests
- CLI changes require integration tests
- Manual testing checklist must pass
- No regression in existing functionality

### Documentation
- Update DEVELOPER.md for architectural changes
- Update README.md for user-facing changes
- Add rustdoc comments for public APIs
- Update task status in `.kiro/specs/cudamgr/tasks.md`

## 🐛 Debugging Common Issues

### Compilation Errors
```bash
# Clear build cache
cargo clean

# Update dependencies
cargo update

# Check for conflicting features
cargo tree -d
```

### Test Failures
```bash
# Run specific failing test
cargo test test_name -- --exact --nocapture

# Run with debug logging
RUST_LOG=debug cargo test test_name

# Check for race conditions
cargo test -- --test-threads=1
```

### Runtime Issues
```bash
# Enable backtraces
RUST_BACKTRACE=1 cargo run -- command

# Debug logging
RUST_LOG=debug cargo run -- command

# Trace all operations
RUST_LOG=trace cargo run -- command
```

## 📚 Additional Resources

- [Rust Book](https://doc.rust-lang.org/book/) - Rust fundamentals
- [Clap Documentation](https://docs.rs/clap/) - CLI argument parsing
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial) - Async programming
- [Tracing Guide](https://tracing.rs/) - Structured logging
- [CUDA Documentation](https://docs.nvidia.com/cuda/) - CUDA toolkit reference