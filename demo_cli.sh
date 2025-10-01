#!/bin/bash

echo "=== CudaMgr CLI Framework Demo ==="
echo

echo "1. Main help:"
cargo run -- --help
echo

echo "2. Doctor command help:"
cargo run -- doctor --help
echo

echo "3. Install command help:"
cargo run -- install --help
echo

echo "4. Testing validation - invalid version (should fail):"
cargo run -- install "invalid@version" 2>&1 | head -3
echo

echo "5. Testing validation - empty version (should fail):"
cargo run -- install "" 2>&1 | head -3
echo

echo "6. Testing validation - invalid logs lines (should fail):"
cargo run -- logs --lines 0 2>&1 | head -3
echo

echo "7. Testing valid command (should show 'not implemented'):"
cargo run -- doctor --verbose 2>&1 | head -3
echo

echo "8. Testing another valid command:"
cargo run -- install "11.8" 2>&1 | head -3
echo

echo "=== CLI Framework Implementation Complete ==="