# Contributing to CudaMgr

Thank you for your interest in contributing to CudaMgr! This document provides guidelines and instructions for contributing.

## 🚀 Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/your-username/cudamgr.git
   cd cudamgr
   ```
3. **Create a branch** for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/your-bug-fix
   ```

## 📋 Development Setup

### Prerequisites

- **Rust 1.70+** - Install from [rustup.rs](https://rustup.rs/)
- **Git** - For version control
- **Cargo** - Comes with Rust installation

### Building the Project

```bash
# Debug build (faster compilation)
cargo build

# Release build (optimized)
cargo build --release
```

### Running Tests

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test '*'

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

## ✅ Pre-Submission Checklist

Before submitting a pull request, ensure:

### Code Quality

- [ ] **Formatting**: Run `cargo fmt` to format your code
- [ ] **Linting**: Run `cargo clippy --all-targets --all-features -- -D warnings` and fix all warnings
- [ ] **Build**: Project builds successfully (`cargo build`)
- [ ] **Tests**: All tests pass (`cargo test`)

### Testing Requirements

- [ ] **Unit Tests**: Added tests for new functionality
- [ ] **Integration Tests**: Updated integration tests if CLI changed
- [ ] **Manual Testing**: Tested the changes manually
- [ ] **No Regressions**: Existing functionality still works

### Documentation

- [ ] **Code Comments**: Added comments for complex logic
- [ ] **README**: Updated README.md if user-facing changes
- [ ] **DEVELOPER.md**: Updated if architectural changes
- [ ] **Changelog**: Documented changes (if applicable)

## 🔍 Code Quality Standards

### Formatting

Always format your code before committing:

```bash
cargo fmt
```

The CI will check formatting automatically. PRs with unformatted code will be rejected.

### Linting

We use Clippy for linting. Run it before submitting:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Fix all warnings before submitting your PR.

### Code Style

- Follow Rust naming conventions (snake_case for functions/variables, PascalCase for types)
- Use meaningful variable and function names
- Keep functions focused and small
- Add documentation comments for public APIs
- Use `thiserror` for error types
- Prefer `Result` over panicking

## 🧪 Testing Guidelines

### Writing Tests

1. **Unit Tests**: Place in `src/**/tests.rs` files
2. **Integration Tests**: Place in `tests/` directory
3. **Test Naming**: Use descriptive names like `test_function_name_scenario`

### Test Examples

```rust
// Unit test example
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_with_valid_input() {
        let result = function("valid_input");
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_with_invalid_input() {
        let result = function("");
        assert!(result.is_err());
    }
}
```

### Running Tests Locally

```bash
# All tests
cargo test

# Specific test file
cargo test --test integration_test

# With output
cargo test -- --nocapture

# Single test
cargo test test_name -- --exact
```

## 📝 Pull Request Process

1. **Update your branch** with the latest changes from main:
   ```bash
   git checkout main
   git pull upstream main
   git checkout your-branch
   git rebase main
   ```

2. **Ensure all checks pass**:
   ```bash
   cargo fmt --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo build
   cargo test
   ```

3. **Push your changes**:
   ```bash
   git push origin your-branch
   ```

4. **Create a Pull Request** on GitHub:
   - Use the PR template
   - Provide a clear description
   - Link related issues
   - Wait for CI to pass

### PR Requirements

- ✅ All CI checks must pass (formatting, linting, build, tests)
- ✅ At least one review approval required
- ✅ No merge conflicts
- ✅ Follows the PR template

## 🐛 Reporting Bugs

When reporting bugs, please include:

1. **Description**: Clear description of the bug
2. **Steps to Reproduce**: Detailed steps to reproduce
3. **Expected Behavior**: What should happen
4. **Actual Behavior**: What actually happens
5. **Environment**: OS, Rust version, etc.
6. **Logs**: Relevant error messages or logs

## 💡 Suggesting Features

When suggesting features:

1. **Use Case**: Explain the problem it solves
2. **Proposed Solution**: Describe your proposed solution
3. **Alternatives**: Consider alternative approaches
4. **Implementation**: If possible, outline how it could be implemented

## 📚 Documentation

### Code Documentation

- Use rustdoc comments for public APIs:
  ```rust
  /// This function does something important.
  ///
  /// # Arguments
  ///
  /// * `input` - The input string to process
  ///
  /// # Returns
  ///
  /// Returns a `Result` containing the processed output
  pub fn process(input: &str) -> Result<String> {
      // ...
  }
  ```

### Updating Documentation

- **README.md**: Update for user-facing changes
- **DEVELOPER.md**: Update for architectural or technical changes
- **Code Comments**: Add inline comments for complex logic

## 🔄 Commit Messages

Write clear, descriptive commit messages:

```
feat: Add GPU detection functionality

- Implement GPU vendor detection
- Add compute capability checking
- Add tests for GPU detection

Fixes #123
```

Common prefixes:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `test:` - Test additions/changes
- `refactor:` - Code refactoring
- `style:` - Code style changes (formatting)
- `chore:` - Maintenance tasks

## 🎯 Development Workflow

1. **Create an issue** (for bugs or features) or check existing issues
2. **Assign yourself** to the issue
3. **Create a branch** from `main`
4. **Make your changes** with tests
5. **Run checks** locally (fmt, clippy, tests)
6. **Commit and push** your changes
7. **Create a PR** and wait for review
8. **Address feedback** and update the PR
9. **Merge** after approval and CI passes

## ❓ Getting Help

- **Questions**: Open a discussion on GitHub
- **Bugs**: Open an issue
- **Features**: Open an issue for discussion first
- **Code Review**: Tag maintainers in your PR

## 📜 License

By contributing, you agree that your contributions will be licensed under the same license as the project (see LICENSE file).

## 🙏 Thank You!

Your contributions make CudaMgr better for everyone. Thank you for taking the time to contribute!

