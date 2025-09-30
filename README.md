# CudaMgr

A cross-platform CUDA version manager built in Rust.

## Project Structure

```
src/
├── main.rs              # Application entry point
├── error.rs             # Error types and handling
├── cli/                 # Command-line interface
│   ├── mod.rs
│   ├── commands.rs      # Command definitions
│   ├── output.rs        # Formatted output utilities
│   └── interactive.rs   # User interaction prompts
├── system/              # System checking and compatibility
│   ├── mod.rs
│   ├── gpu.rs           # GPU detection
│   ├── driver.rs        # Driver detection
│   ├── compiler.rs      # Compiler detection
│   ├── distro.rs        # Distribution detection
│   ├── storage.rs       # Storage checking
│   └── security.rs      # Security validation
├── install/             # Installation management
│   ├── mod.rs
│   ├── downloader.rs    # Package downloading
│   ├── installer.rs     # Platform-specific installation
│   ├── validator.rs     # Installation validation
│   └── cleanup.rs       # Cleanup utilities
├── version/             # Version management
│   ├── mod.rs
│   ├── registry.rs      # Version registry
│   ├── switcher.rs      # Version switching
│   └── resolver.rs      # Version resolution
└── config/              # Configuration management
    ├── mod.rs
    ├── env.rs           # Environment variables
    ├── path.rs          # PATH management
    ├── symlink.rs       # Symlink management
    └── shell.rs         # Shell configuration
```

## Available Commands

- `cudamgr doctor` - Check system compatibility
- `cudamgr install <version>` - Install CUDA version
- `cudamgr use <version>` - Switch to CUDA version
- `cudamgr list` - List installed versions
- `cudamgr uninstall <version>` - Uninstall CUDA version
- `cudamgr logs` - View logs

## Development

```bash
# Build the project
cargo build

# Run with logging
RUST_LOG=info cargo run -- doctor

# Check code
cargo check

# Run tests
cargo test
```

## Dependencies

- `clap` - Command-line argument parsing
- `tokio` - Async runtime
- `serde` - Serialization
- `thiserror` - Error handling
- `reqwest` - HTTP client
- `tracing` - Logging
- `chrono` - Date/time handling