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

# Prompt for version bump
echo -e "\n${GREEN}Current version from Cargo.toml:${NC}"
current_version=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
echo "Current version: $current_version"

echo -e "\n${GREEN}Would you like to bump the version? (y/n)${NC}"
read -r response

if [[ "$response" =~ ^[Yy]$ ]]; then
    echo -e "\n${GREEN}Enter new version:${NC}"
    read -r new_version
    
    # Update version in Cargo.toml
    sed -i.bak "s/^version = \"$current_version\"/version = \"$new_version\"/" Cargo.toml
    rm Cargo.toml.bak
    
    # Git commands
    git add Cargo.toml
    git commit -m "chore: bump version to $new_version"
    git tag -a "v$new_version" -m "Version $new_version"
    
    echo -e "\n${GREEN}Version bumped to $new_version${NC}"
fi

# Final confirmation before publish
echo -e "\n${GREEN}Ready to publish. Continue? (y/n)${NC}"
read -r publish_confirm

if [[ "$publish_confirm" =~ ^[Yy]$ ]]; then
    echo -e "\n${GREEN}Publishing to crates.io...${NC}"
    cargo publish --allow-dirty
    
    # Push to git if version was bumped
    if [[ "$response" =~ ^[Yy]$ ]]; then
        git push && git push --tags
    fi
    
    echo -e "\n${GREEN}Successfully published swarms-rs!${NC}"
else
    echo -e "\n${RED}Publish cancelled${NC}"
    exit 1
fi
