#!/bin/bash

# Exit on error
set -e

# Run formatting script
echo "Formatting code..."
./format_code.sh

# Run publishing script
echo "Publishing code..."
./publish.sh

echo "Done!"