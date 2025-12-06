# Branch Protection Settings

This document describes the recommended branch protection settings for this repository. These settings ensure that all code changes go through proper review and testing before being merged.

## Required Settings

To enable branch protection, go to:
**Settings → Branches → Branch protection rules → Add rule**

### Protected Branch: `main` (and optionally `develop`)

#### Required Settings:

1. **Require a pull request before merging**
   - ✅ Require approvals: **1** (or more)
   - ✅ Dismiss stale pull request approvals when new commits are pushed
   - ✅ Require review from Code Owners (if CODEOWNERS file exists)

2. **Require status checks to pass before merging**
   - ✅ Require branches to be up to date before merging
   - ✅ Required status checks:
     - `Format Check` (fmt job)
     - `Clippy Lint` (clippy job)
     - `Build (ubuntu-latest)` (build job)
     - `Build (windows-latest)` (build job)
     - `Build (macos-latest)` (build job)
     - `Test Suite (ubuntu-latest)` (test job)
     - `Test Suite (windows-latest)` (test job)
     - `Test Suite (macos-latest)` (test job)
     - `All Checks Passed` (ci-success job)

3. **Require conversation resolution before merging**
   - ✅ Require all conversations on code to be resolved

4. **Do not allow bypassing the above settings**
   - ✅ Do not allow bypassing the above settings (recommended for security)

#### Optional but Recommended:

- **Require linear history** - Ensures a clean git history
- **Include administrators** - Applies rules to admins too
- **Restrict who can push to matching branches** - Limit direct pushes to main

## How It Works

When a PR is created:

1. **CI Workflow Runs**: The GitHub Actions workflow automatically runs on the PR
2. **All Checks Must Pass**: Formatting, linting, building, and tests must all pass
3. **Review Required**: At least one approval is required
4. **No Conflicts**: Branch must be up to date with main
5. **Merge Allowed**: Only after all conditions are met

## Status Check Details

The CI workflow creates the following status checks:

- **Format Check**: Ensures code is properly formatted with `cargo fmt`
- **Clippy Lint**: Catches common mistakes and enforces best practices
- **Build**: Verifies the project compiles on Linux, Windows, and macOS
- **Test Suite**: Runs all tests on all platforms
- **All Checks Passed**: Final verification that all jobs succeeded

All of these must pass before a PR can be merged.

## Setting Up Branch Protection

1. Go to your repository on GitHub
2. Click **Settings** → **Branches**
3. Click **Add rule** or edit existing rule for `main`
4. Configure the settings as described above
5. Save the rule

## Troubleshooting

### "Required status checks are pending"

This means the CI workflow is still running. Wait for it to complete.

### "Required status checks must pass"

One or more checks failed. Check the Actions tab to see which job failed and fix the issues.

### "This branch is out of date"

Update your branch with the latest changes from main:
```bash
git checkout main
git pull
git checkout your-branch
git rebase main
git push --force-with-lease
```

