#!/bin/bash
# Measure baseline binary sizes and optionally update thresholds
# Usage: ./measure-baseline-sizes.sh [--update-config]

set -e

# Colors for output
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

TARGET_DIR="${TARGET_DIR:-target/release}"
CONFIG_FILE="${SIZE_CONFIG_FILE:-size-thresholds.json}"
UPDATE_CONFIG=false

# Parse arguments
if [ "$1" = "--update-config" ]; then
    UPDATE_CONFIG=true
fi

# Function to format bytes to human readable
format_bytes() {
    local bytes=$1
    if [ $bytes -lt 1024 ]; then
        echo "${bytes}B"
    elif [ $bytes -lt 1048576 ]; then
        echo "$(awk "BEGIN {printf \"%.2f\", $bytes/1024}")KB"
    elif [ $bytes -lt 1073741824 ]; then
        echo "$(awk "BEGIN {printf \"%.2f\", $bytes/1048576}")MB"
    else
        echo "$(awk "BEGIN {printf \"%.2f\", $bytes/1073741824}")GB"
    fi
}

echo -e "${BLUE}=== Measuring Baseline Binary Sizes ===${NC}"
echo "Target directory: $TARGET_DIR"
echo ""

# Check if target directory exists
if [ ! -d "$TARGET_DIR" ]; then
    echo -e "${YELLOW}Warning: Target directory not found: $TARGET_DIR${NC}"
    echo "Building release binaries..."
    cargo build --release
    echo ""
fi

# Measure binaries
echo -e "${BLUE}Binary Sizes:${NC}"
echo ""

MEASUREMENTS="{"

# Measure glassbox-cli
if [ -f "$TARGET_DIR/glassbox-cli" ]; then
    size=$(stat -c%s "$TARGET_DIR/glassbox-cli" 2>/dev/null || stat -f%z "$TARGET_DIR/glassbox-cli" 2>/dev/null)
    human=$(format_bytes $size)
    echo "glassbox-cli: $human ($size bytes)"
    MEASUREMENTS="$MEASUREMENTS\"glassbox-cli\": $size,"
fi

# Measure watch-demo
if [ -f "$TARGET_DIR/watch-demo" ]; then
    size=$(stat -c%s "$TARGET_DIR/watch-demo" 2>/dev/null || stat -f%z "$TARGET_DIR/watch-demo" 2>/dev/null)
    human=$(format_bytes $size)
    echo "watch-demo: $human ($size bytes)"
    MEASUREMENTS="$MEASUREMENTS\"watch-demo\": $size,"
fi

# Measure library
lib_file=$(find "$TARGET_DIR" -name "libglassbox.rlib" -o -name "libglassbox-*.rlib" | head -1)
if [ -n "$lib_file" ]; then
    size=$(stat -c%s "$lib_file" 2>/dev/null || stat -f%z "$lib_file" 2>/dev/null)
    human=$(format_bytes $size)
    echo "libglassbox.rlib: $human ($size bytes)"
    MEASUREMENTS="$MEASUREMENTS\"libglassbox.rlib\": $size"
else
    # Remove trailing comma if library not found
    MEASUREMENTS="${MEASUREMENTS%,}"
fi

MEASUREMENTS="$MEASUREMENTS}"

echo ""

# Update config if requested
if [ "$UPDATE_CONFIG" = true ]; then
    if ! command -v jq &> /dev/null; then
        echo -e "${YELLOW}Warning: jq is required to update config${NC}"
        exit 1
    fi
    
    echo -e "${BLUE}Updating configuration file: $CONFIG_FILE${NC}"
    
    # Create backup
    cp "$CONFIG_FILE" "$CONFIG_FILE.backup"
    
    # Update thresholds with 20% buffer
    NEW_CONFIG=$(cat "$CONFIG_FILE")
    
    for binary in glassbox-cli watch-demo libglassbox.rlib; do
        size=$(echo "$MEASUREMENTS" | jq -r ".[\"$binary\"] // empty")
        if [ -n "$size" ] && [ "$size" != "null" ]; then
            # Add 20% buffer for threshold
            threshold=$(awk "BEGIN {printf \"%.0f\", $size * 1.2}")
            threshold_human=$(format_bytes $threshold)
            
            NEW_CONFIG=$(echo "$NEW_CONFIG" | jq \
                --arg binary "$binary" \
                --argjson threshold "$threshold" \
                --arg threshold_human "$threshold_human" \
                '.thresholds[$binary].max_size_bytes = $threshold | 
                 .thresholds[$binary].max_size_human = $threshold_human')
            
            echo "  Updated $binary: $threshold_human"
        fi
    done
    
    echo "$NEW_CONFIG" > "$CONFIG_FILE"
    echo ""
    echo -e "${GREEN}✅ Configuration updated successfully${NC}"
    echo "Backup saved to: $CONFIG_FILE.backup"
else
    echo -e "${YELLOW}To update thresholds based on these measurements, run:${NC}"
    echo "  ./scripts/measure-baseline-sizes.sh --update-config"
fi

echo ""
echo -e "${GREEN}✅ Baseline measurement complete${NC}"
