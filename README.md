# CudaMgr

A cross-platform CUDA version manager that simplifies installing, managing, and switching between different CUDA toolkit versions.

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- Git

### Installation

```bash
git clone https://github.com/cudamgr/cudamgr
cd cudamgr
cargo build --release
```

### Basic Usage

```bash
# Check system compatibility
cargo run -- doctor --verbose

# Get help for any command
cargo run -- --help
cargo run -- install --help
```

## 📋 Commands

| Command | Description | Status |
|---------|-------------|--------|
| `doctor` | Check system compatibility for CUDA | 🚧 Coming Soon |
| `install <version>` | Install a specific CUDA version | 🚧 Coming Soon |
| `use <version>` | Switch to a CUDA version | 🚧 Coming Soon |
| `list` | List installed and available versions | 🚧 Coming Soon |
| `uninstall <version>` | Remove a CUDA version | 🚧 Coming Soon |
| `logs` | View installation logs | 🚧 Coming Soon |

### Examples

```bash
# Check if your system supports CUDA
cudamgr doctor --verbose

# Install CUDA 11.8 (coming soon)
cudamgr install 11.8 --force

# Switch to CUDA 12.0 (coming soon)
cudamgr use 12.0

# List all versions (coming soon)
cudamgr list --available
```

## 🧪 Testing the CLI

### Quick Test

```bash
# Test help system
cargo run -- --help
cargo run -- doctor --help

# Test command validation
cargo run -- doctor
cargo run -- install 11.8  # Should show "not implemented"
```

### Full Test Suite

```bash
# Run all tests
cargo test

# Test CLI functionality
cargo test --lib cli

# Test integration
cargo test --test cli_integration_test
```

### Manual Testing

Try these commands to verify everything works:

```bash
# Valid commands (should show "not implemented" message)
cargo run -- doctor
cargo run -- install 11.8
cargo run -- use 12.0 --install
cargo run -- list --available
cargo run -- uninstall 11.8 --yes
cargo run -- logs --lines 50

# Invalid commands (should show validation errors)
cargo run -- install ""
cargo run -- install "bad-version!"
cargo run -- logs --lines 0
```

**Expected Results:**
- ✅ Help commands exit with code 0
- ✅ Valid commands show "not implemented" and exit with code 1
- ✅ Invalid commands show validation errors and exit with code 1

## 🚧 Development Status

**Current Phase: CLI Framework** ✅ **COMPLETED**
- ✅ Command parsing and validation
- ✅ Help system and error handling
- ✅ Interactive prompts and progress indicators
- ✅ Comprehensive test coverage

**Next Phase: System Detection** 🚧 **IN PROGRESS**
- 🚧 GPU and driver detection
- 🚧 OS and package manager detection
- 🚧 Compatibility validation

## 📖 Documentation

- **[DEVELOPER.md](DEVELOPER.md)** - Technical details, architecture, and development guide
- **[.kiro/specs/cudamgr/](/.kiro/specs/cudamgr/)** - Detailed specifications and requirements

## 🤝 Contributing

1. Check [DEVELOPER.md](DEVELOPER.md) for technical setup
2. Review current tasks in `.kiro/specs/cudamgr/tasks.md`
3. Run `cargo test` to ensure everything works
4. Test CLI commands manually before submitting changes

