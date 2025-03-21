#!/bin/bash

# Exit on error
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color


# Format all code
echo -e "\n${GREEN}Formatting code...${NC}"
cargo fmt --all

# Build documentation
echo -e "\n${GREEN}Building documentation...${NC}"
cargo doc --no-deps

# Dry run to check for any publishing issues
echo -e "\n${GREEN}Performing dry run...${NC}"
cargo publish --dry-run --allow-dirty

# Publish the package
echo -e "\n${GREEN}Publishing package...${NC}"
cargo publish --allow-dirty

echo -e "\n${GREEN}Publish complete!${NC}"


echo -e "\n${GREEN}Done!${NC}"