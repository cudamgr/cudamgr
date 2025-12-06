# Pre-submission check script for Windows PowerShell
# Run this before submitting a PR to ensure all checks pass

$ErrorActionPreference = "Stop"

Write-Host "🔍 Running pre-submission checks..." -ForegroundColor Cyan
Write-Host ""

# Function to print status
function Print-Status {
    param(
        [int]$ExitCode,
        [string]$Message
    )
    
    if ($ExitCode -eq 0) {
        Write-Host "✅ $Message" -ForegroundColor Green
    } else {
        Write-Host "❌ $Message" -ForegroundColor Red
        exit 1
    }
}

# Check if cargo is installed
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Cargo is not installed. Please install Rust from https://rustup.rs/" -ForegroundColor Red
    exit 1
}

# 1. Format check
Write-Host "📝 Checking code formatting..." -ForegroundColor Yellow
cargo fmt --all -- --check
Print-Status $LASTEXITCODE "Formatting check passed"

# 2. Clippy lint
Write-Host "🔧 Running Clippy linter..." -ForegroundColor Yellow
cargo clippy --all-targets --all-features -- -D warnings
Print-Status $LASTEXITCODE "Clippy lint passed"

# 3. Build check
Write-Host "🔨 Building project..." -ForegroundColor Yellow
cargo build --release
Print-Status $LASTEXITCODE "Build succeeded"

# 4. Run tests
Write-Host "🧪 Running tests..." -ForegroundColor Yellow
cargo test --verbose
Print-Status $LASTEXITCODE "All tests passed"

Write-Host ""
Write-Host "🎉 All checks passed! You're ready to submit your PR." -ForegroundColor Green

