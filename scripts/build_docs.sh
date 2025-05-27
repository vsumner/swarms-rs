#!/bin/bash

# Exit on error
set -e

# Colors for output
GREEN='\033[0;32m'
NC='\033[0m' # No Color

echo -e "${GREEN}Building swarms-rs documentation...${NC}"

# Ensure we're in the project root directory
cd "$(dirname "$0")/.."

# Clean any existing documentation
rm -rf target/doc

# Build the documentation with the best flags
echo -e "${GREEN}Building Rust documentation...${NC}"
cargo doc --no-deps --document-private-items --all-features --open

# The flags used:
# --no-deps: Only document your project's code, not dependencies
# --document-private-items: Include private items in documentation
# --all-features: Build docs with all features enabled
# --open: Open the docs in your default browser

echo -e "${GREEN}Documentation built successfully!${NC}"
echo -e "You can find the documentation at: file://$(pwd)/target/doc/index.html"