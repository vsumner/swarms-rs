#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Starting code formatting and cleaning...${NC}"

# Change to swarms-rs directory
cd swarms-rs || exit

# Check if rustfmt is installed
if ! command -v rustfmt &> /dev/null; then
    echo "rustfmt is not installed. Installing..."
    rustup component add rustfmt
fi

# Format all Rust code
echo -e "${BLUE}Formatting Rust code...${NC}"
cargo fmt

# Clean the project
echo -e "${BLUE}Cleaning the project...${NC}"
cargo clean

# Check for any warnings or errors
echo -e "${BLUE}Checking for warnings and errors...${NC}"
cargo check

echo -e "${GREEN}Formatting and cleaning completed!${NC}" 