#!/bin/bash

# Generate Python types from Rust protocol library
# This script ensures consistency between Rust and Python implementations

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CODEGEN_DIR="$PROJECT_ROOT/codegen"
PYTHON_DIR="$PROJECT_ROOT/pc/python/plugin_host"
HASH_CHECK_SCRIPT="$SCRIPT_DIR/check-python-types-hash.sh"

# Check if regeneration is needed
if [ -f "$HASH_CHECK_SCRIPT" ]; then
    echo "🔐 Checking if regeneration is needed..."
    if "$HASH_CHECK_SCRIPT"; then
        echo "🔄 Protocol changes detected, regenerating..."
    else
        echo "✅ Generated types are up to date, skipping regeneration"
        echo "💡 Use --force flag to regenerate anyway"
        if [ "$1" != "--force" ]; then
            exit 0
        fi
        echo "🔄 Force regeneration requested..."
    fi
fi

echo "🔄 Generating Python types from Rust protocol library..."

# Change to codegen directory
cd "$CODEGEN_DIR"

# Run tests first to validate codegen
echo "🧪 Running codegen tests..."
if ! cargo test --quiet > /dev/null 2>&1; then
    echo "❌ Tests failed! Aborting generation."
    exit 1
fi
echo "✅ All tests passed!"

# Build the codegen tool if needed
echo "🔨 Building code generator..."
cargo build --release --bin generate-python

# Generate Python code
echo "🐍 Generating Python types..."
cargo run --release --bin generate-python -- \
    --protocol-path "$PROJECT_ROOT/protocol/src" \
    --output-dir "$PYTHON_DIR"

# Check if generated file exists
GENERATED_FILE="$PYTHON_DIR/generated_types.py"
if [ -f "$GENERATED_FILE" ]; then
    echo "✅ Generated: $GENERATED_FILE"
    
    # Display summary
    echo ""
    echo "📊 Generation Summary:"
    echo "   - Constants: $(grep -c "^[A-Z_]* = " "$GENERATED_FILE" || echo "0")"
    echo "   - Enums: $(grep -c "^class.*Enum" "$GENERATED_FILE" || echo "0")"
    echo "   - Structs: $(grep -c "^class.*:" "$GENERATED_FILE" | grep -c "@attr.s" || echo "0")"
    
    # Show generated file size
    FILE_SIZE=$(wc -l < "$GENERATED_FILE")
    echo "   - Lines: $FILE_SIZE"
    
    echo ""
    echo "💡 Next steps:"
    echo "   1. Review the generated file: $GENERATED_FILE"
    echo "   2. Replace or merge with existing types.py"
    echo "   3. Test your Python code with the new types"
    echo "   4. Commit the changes if everything looks good"
    
else
    echo "❌ Failed to generate Python types"
    exit 1
fi