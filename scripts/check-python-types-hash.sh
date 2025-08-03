#!/bin/bash

# Check if Python types need regeneration by comparing protocol hash
# Returns 0 if regeneration is needed, 1 if current

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CODEGEN_DIR="$PROJECT_ROOT/codegen"
PYTHON_DIR="$PROJECT_ROOT/pc/python/plugin_host"
GENERATED_FILE="$PYTHON_DIR/generated_types.py"

# Function to calculate current protocol hash
calculate_current_hash() {
    # Extract hash by running just the hash calculation part
    python3 -c "
import hashlib
import os
import glob
protocol_src = '$PROJECT_ROOT/protocol/src'
# Get all .rs files and sort them for consistent hash
rs_files = sorted(glob.glob(os.path.join(protocol_src, '*.rs')))
hasher = hashlib.sha256()
for filepath in rs_files:
    with open(filepath, 'r') as f:
        hasher.update(f.read().encode())
print(hasher.hexdigest())
"
}

# Function to extract hash from generated file
extract_existing_hash() {
    if [ ! -f "$GENERATED_FILE" ]; then
        echo ""
        return
    fi
    
    grep '^PROTOCOL_HASH = ' "$GENERATED_FILE" 2>/dev/null | sed 's/PROTOCOL_HASH = "\(.*\)"/\1/' || echo ""
}

# Calculate hashes
echo "🔐 Checking protocol hash..."
CURRENT_HASH=$(calculate_current_hash)
EXISTING_HASH=$(extract_existing_hash)

echo "Current protocol hash:  $CURRENT_HASH"
echo "Existing generated hash: $EXISTING_HASH"

if [ "$CURRENT_HASH" = "$EXISTING_HASH" ] && [ -n "$EXISTING_HASH" ]; then
    echo "✅ Generated types are up to date"
    exit 1  # No regeneration needed
else
    echo "🔄 Generated types need to be regenerated"
    exit 0  # Regeneration needed
fi