#!/bin/bash
# publish_crate.sh - Script to prepare and publish a Rust crate to crates.io

set -e  # Exit immediately if a command exits with a non-zero status

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to print step information
print_step() {
    echo -e "\n${YELLOW}==== $1 ====${NC}"
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check if cargo is installed
if ! command_exists cargo; then
    echo -e "${RED}Error: cargo is not installed. Please install Rust and Cargo first.${NC}"
    exit 1
fi

# Parse command line arguments
SKIP_TESTS=0
SKIP_LOGIN=0
DRY_RUN=0

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --skip-tests) SKIP_TESTS=1 ;;
        --skip-login) SKIP_LOGIN=1 ;;
        --dry-run) DRY_RUN=1 ;;
        --help) 
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  --skip-tests    Skip running tests"
            echo "  --skip-login    Skip cargo login step"
            echo "  --dry-run       Run all checks but don't publish"
            echo "  --help          Show this help message"
            exit 0
            ;;
        *) echo "Unknown parameter: $1"; exit 1 ;;
    esac
    shift
done

# Step 1: Verify Cargo.toml has all required fields
print_step "Verifying Cargo.toml"

required_fields=("name" "version" "description" "license" "repository" "readme")
missing_fields=()

for field in "${required_fields[@]}"; do
    if ! grep -q "^$field\s*=" Cargo.toml; then
        missing_fields+=("$field")
    fi
done

if [ ${#missing_fields[@]} -ne 0 ]; then
    echo -e "${RED}Error: The following required fields are missing in Cargo.toml:${NC}"
    for field in "${missing_fields[@]}"; do
        echo "  - $field"
    done
    exit 1
fi

# Step 4: Run tests
if [ $SKIP_TESTS -eq 0 ]; then
    print_step "Running tests"
    cargo test
    echo -e "${GREEN}All tests passed.${NC}"
    
    print_step "Running clippy"
    cargo clippy -- -D warnings
    echo -e "${GREEN}Clippy checks passed.${NC}"
    
    print_step "Checking formatting"
    cargo fmt -- --check
    echo -e "${GREEN}Formatting checks passed.${NC}"
else
    echo -e "${YELLOW}Skipping tests as requested.${NC}"
fi

# Step 5: Build documentation
print_step "Building documentation"
cargo doc --no-deps
echo -e "${GREEN}Documentation built successfully.${NC}"

# Step 6: Login to crates.io if needed
if [ $SKIP_LOGIN -eq 0 ]; then
    print_step "Logging in to crates.io"
    echo -e "${YELLOW}Please enter your crates.io API token when prompted.${NC}"
    echo -e "${YELLOW}(You can get this from https://crates.io/me)${NC}"
    cargo login
    echo -e "${GREEN}Login successful.${NC}"
else
    echo -e "${YELLOW}Skipping login as requested.${NC}"
fi

# Step 7: Check what will be included in the package
print_step "Checking package contents"
cargo package --list
echo -e "${GREEN}Package contents listed above.${NC}"

# Step 8: Create the package (dry run)
print_step "Creating package (dry run)"
cargo package
echo -e "${GREEN}Package created successfully.${NC}"

# Step 9: Publish the crate
if [ $DRY_RUN -eq 0 ]; then
    print_step "Publishing crate to crates.io"
    echo -e "${YELLOW}Are you sure you want to publish this crate? (y/n)${NC}"
    read -r confirm
    if [[ $confirm == [yY] || $confirm == [yY][eE][sS] ]]; then
        cargo publish
        echo -e "${GREEN}Crate published successfully!${NC}"
        
        # Get crate name and version for the final message
        crate_name=$(grep "^name" Cargo.toml | head -1 | cut -d '"' -f 2 | cut -d "'" -f 2)
        crate_version=$(grep "^version" Cargo.toml | head -1 | cut -d '"' -f 2 | cut -d "'" -f 2)
        
        echo -e "${GREEN}Your crate is now available at: https://crates.io/crates/$crate_name/$crate_version${NC}"
        echo -e "${GREEN}Documentation will be available at: https://docs.rs/$crate_name/$crate_version${NC}"
    else
        echo -e "${YELLOW}Publication cancelled.${NC}"
    fi
else
    echo -e "${YELLOW}Dry run complete. Use without --dry-run to publish.${NC}"
fi

echo -e "\n${GREEN}Script completed successfully!${NC}"