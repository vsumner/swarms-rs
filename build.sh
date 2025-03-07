#!/bin/bash
set -e  # Exit immediately if any command fails.

echo "=== Starting build process from scratch ==="

# 1. Remove any existing virtual environment.
if [ -d ".venv" ]; then
    echo "Deleting existing virtual environment (.venv)..."
    rm -rf .venv
fi

# 2. Create a new virtual environment with Python 3.12.
echo "Creating new virtual environment with Python 3.12..."
python3.12 -m venv .venv

# 3. Activate the virtual environment.
echo "Activating virtual environment..."
source .venv/bin/activate

# 4. Upgrade pip, setuptools, and wheel.
echo "Upgrading pip, setuptools, and wheel..."
/usr/local/bin/python3.12 -m pip install --upgrade pip setuptools wheel

# 5. Install maturin (required to build the package).
echo "Installing maturin..."
/usr/local/bin/python3.12 -m pip install maturin

# 6. Build and install the Rust extension into the virtual environment.
echo "Building and installing Rust extension with maturin..."
maturin develop

# 7. Build the final package (wheel) using pip.
echo "Building wheel package..."
pip wheel .

echo "=== Build process completed successfully! ==="
