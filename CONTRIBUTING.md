# Contributing to CudaMgr

Thank you for your interest in contributing to CudaMgr! We welcome contributions from the community.

## Contribution Workflow

Please follow this exact process to contribute:

### 1. File an Issue
Before starting work, ensure there is an issue created for the bug or feature you want to work on. Note the **Issue Number**.

### 2. Fork the Repository
Fork the `cudamgr` repository to your own GitHub account.

### 3. Create a Branch
Create a new branch in your fork. **You MUST name the branch using the issue number** you are addressing.

```bash
# Syntax: issue-<number> or issue-<number>-<short-description>
git checkout -b issue-123
# OR
git checkout -b issue-123-fix-installer
```

### 4. Make Changes
Implement your changes in this branch.

### 5. Run Checks (Mandatory)
Before committing, you **MUST** run the following checks locally to ensure your code matches our quality standards.

```bash
# 1. Format Check
cargo fmt -- --check

# 2. Lint Check (Clippy)
# We treat warnings as errors!
cargo clippy -- -D warnings

# 3. Compliance Check (Licenses & Bans)
cargo deny check
```

**If any of these fail, fix the errors before proceeding.**

### 6. Create Pull Request
1.  Push your branch to your fork:
    ```bash
    git push origin issue-123
    ```
2.  Open a Pull Request (PR) from your branch to the `main` branch of the upstream `cudamgr` repository.
3.  Reference the issue in your PR description (e.g., "Closes #123").

## Code Style & Standards
- **Formatting**: We use standard `rustfmt`.
- **Linting**: We enforce `clippy` pedantic checks where reasonable.
- **Licenses**: We use `cargo-deny` to ensure all dependencies use compatible open-source licenses (MIT, Apache-2.0, BSD-3-Clause, MPL-2.0, Unicode-3.0).
