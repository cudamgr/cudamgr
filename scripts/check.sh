#!/bin/bash
# Pre-submission check script
# Run this before submitting a PR to ensure all checks pass

set -e

echo "🔍 Running pre-submission checks..."
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
    else
        echo -e "${RED}❌ $2${NC}"
        exit 1
    fi
}

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Cargo is not installed. Please install Rust from https://rustup.rs/${NC}"
    exit 1
fi

# 1. Format check
echo "📝 Checking code formatting..."
cargo fmt --all -- --check
print_status $? "Formatting check passed"

# 2. Clippy lint
echo "🔧 Running Clippy linter..."
cargo clippy --all-targets --all-features -- -D warnings
print_status $? "Clippy lint passed"

# 3. Build check
echo "🔨 Building project..."
cargo build --release
print_status $? "Build succeeded"

# 4. Run tests
echo "🧪 Running tests..."
cargo test --verbose
print_status $? "All tests passed"

echo ""
echo -e "${GREEN}🎉 All checks passed! You're ready to submit your PR.${NC}"

