# CudaMgr
[![Discord](https://img.shields.io/badge/Discord-Join%20Community-7289DA?style=flat&logo=discord&logoColor=white)](https://discord.gg/Kr7NdB5Qvu)

A cross-platform CUDA version manager that simplifies installing, managing, and switching between different CUDA toolkit versions.

<img width="500" height="500" alt="image" src="https://github.com/user-attachments/assets/0710e86a-e1ff-47ca-a302-db90eb9a3213" />

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

**Current Phase: System Detection** 🚧 **IN PROGRESS**
- ✅ CLI Framework & Command Parsing
- ✅ GPU & Driver Detection (Refactored to use centralized Registry)
- ✅ Windows Visual Studio Detection
- ✅ WSL Environment Detection
- 🚧 Dynamic Manifest Synchronization (Currently using built-in registry)
- 🚧 Installation Logic (Download/Install/Verify)

## 🔮 Roadmap & Missing Features

The following features are industry standard but currently **missing** or planned for future releases:

1.  **Auto-Update / Self-Update**: `cudamgr self-update` to update the CLI binary itself.
2.  **Remote Manifest Sync**: Fetching the latest GPU/Driver/CUDA map from a remote JSON source instead of the built-in static registry.
3.  **Visual Studio Integration**: Better integration with VS Installer to ensure specific workloads (Desktop C++) are present before CUDA install.
4.  **Deep Learning Libraries**: Installing `cuDNN` and `TensorRT` alongside CUDA.
5.  **Checksum Verification**: Validating SHA256 sums of downloaded installers.
6.  **Resumable Downloads**: Support for resuming interrupted large downloads.
7.  **Proxy Support**: Explicit configuration for corporate proxies.

## 📖 Documentation

- **[DEVELOPER.md](DEVELOPER.md)** - Technical details, architecture, and development guide

## 🤝 Contributing

1. Check [DEVELOPER.md](DEVELOPER.md) for technical setup
2. Run `cargo test` to ensure everything works
3. Test CLI commands manually before submitting changes

