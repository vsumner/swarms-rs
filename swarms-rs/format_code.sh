#!/bin/bash

# format.sh - Script to format and lint Rust code in the swarms-rs project

set -e  # Exit on error

echo "🔍 Running code formatting and linting tools..."

# Create necessary configuration files if they don't exist
if [ ! -f rustfmt.toml ]; then
  echo "Creating rustfmt.toml configuration file..."
  cat > rustfmt.toml << EOF
# Basic formatting
max_width = 100
hard_tabs = false
tab_spaces = 4
newline_style = "Auto"
use_small_heuristics = "Default"

# Imports handling
imports_indent = "Block"
imports_layout = "Mixed"
reorder_imports = true
group_imports = "StdExternalCrate"

# Code structure
reorder_modules = true
match_block_trailing_comma = true
trailing_semicolon = true
trailing_comma = "Vertical"
edition = "2021"
format_code_in_doc_comments = true
format_macro_matchers = true
format_macro_bodies = true
format_strings = true
normalize_comments = true
normalize_doc_attributes = true

# Spacing and alignment
binop_separator = "Front"
brace_style = "SameLineWhere"
control_brace_style = "AlwaysSameLine"
empty_item_single_line = true
fn_single_line = false
where_single_line = false
indent_style = "Block"
spaces_around_ranges = false
struct_field_align_threshold = 0
struct_lit_single_line = true
EOF
  echo "✅ rustfmt.toml created"
fi

if [ ! -f clippy.toml ]; then
  echo "Creating clippy.toml configuration file..."
  cat > clippy.toml << EOF
# Clippy configuration
cognitive-complexity-threshold = 30
too-many-arguments-threshold = 10
type-complexity-threshold = 500
EOF
  echo "✅ clippy.toml created"
fi

# Step 1: Format code with rustfmt
echo "🔄 Formatting code with rustfmt..."
cargo fmt
echo "✅ Code formatting complete"

# Step 2: Run clippy for linting
echo "🔄 Running clippy for linting..."
cargo clippy -- -W clippy::all -W clippy::pedantic -W clippy::nursery -D warnings
echo "✅ Linting complete"

# Step 3: Check for unused dependencies
echo "🔄 Checking for unused dependencies..."
cargo install cargo-udeps --locked 2>/dev/null || echo "cargo-udeps already installed"
cargo +nightly udeps
echo "✅ Dependency check complete"

# Step 4: Run tests
echo "🔄 Running tests..."
cargo test
echo "✅ Tests complete"

# Step 5: Build documentation
echo "🔄 Building documentation..."
cargo doc --no-deps
echo "✅ Documentation built"


echo "✨ All formatting and linting tasks completed successfully! ✨"

# Run the script
# ./format_code.sh