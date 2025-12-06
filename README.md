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
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Contributing guidelines and development workflow

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

### Quick Start for Contributors

1. Fork the repository and clone your fork
2. Create a branch for your changes: `git checkout -b feature/your-feature`
3. Make your changes and add tests
4. Run the pre-submission checks:
   ```bash
   # Linux/macOS
   ./scripts/check.sh
   
   # Windows PowerShell
   .\scripts\check.ps1
   ```
5. Submit a pull request

### Pre-Submission Checklist

Before submitting a PR, ensure:
- ✅ Code is formatted: `cargo fmt`
- ✅ No linting errors: `cargo clippy --all-targets --all-features -- -D warnings`
- ✅ Project builds: `cargo build`
- ✅ All tests pass: `cargo test`

All PRs must pass CI checks (formatting, linting, building, and tests on Linux, Windows, and macOS) before they can be merged.

## 🔄 CI/CD

This project uses GitHub Actions for continuous integration. Every pull request automatically runs:

- **Format Check**: Ensures code follows Rust formatting standards
- **Clippy Lint**: Catches common mistakes and enforces best practices
- **Build Verification**: Compiles on Linux, Windows, and macOS
- **Test Suite**: Runs all tests on all platforms
- **Security Audit**: Checks for known vulnerabilities

All checks must pass before a PR can be merged. See [.github/workflows/ci.yml](.github/workflows/ci.yml) for details.

