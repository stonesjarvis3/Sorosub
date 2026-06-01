#!/bin/bash
# Binary size tracking script for Glassbox project
# This script measures binary sizes and compares them against configured thresholds

set -e

# Colors for output
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
CONFIG_FILE="${SIZE_CONFIG_FILE:-size-thresholds.json}"
TARGET_DIR="${TARGET_DIR:-target/release}"
OUTPUT_FILE="${OUTPUT_FILE:-size-report.json}"

# Check if jq is available
if ! command -v jq &> /dev/null; then
    echo -e "${RED}Error: jq is required but not installed${NC}"
    echo "Install with: sudo apt-get install jq (Ubuntu/Debian) or brew install jq (macOS)"
    exit 1
fi

# Check if config file exists
if [ ! -f "$CONFIG_FILE" ]; then
    echo -e "${RED}Error: Configuration file not found: $CONFIG_FILE${NC}"
    exit 1
fi

# Check if target directory exists
if [ ! -d "$TARGET_DIR" ]; then
    echo -e "${RED}Error: Target directory not found: $TARGET_DIR${NC}"
    echo "Please build the project first with: cargo build --release"
    exit 1
fi

echo -e "${BLUE}=== Binary Size Check ===${NC}"
echo "Configuration: $CONFIG_FILE"
echo "Target directory: $TARGET_DIR"
echo ""

# Initialize results
TOTAL_CHECKS=0
PASSED_CHECKS=0
WARNINGS=0
FAILURES=0
RESULTS="[]"

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

# Function to check a single binary
check_binary() {
    local binary_name=$1
    local binary_path=$2
    local max_size=$3
    local warn_percent=$4
    local description=$5
    
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    
    if [ ! -f "$binary_path" ]; then
        echo -e "${YELLOW}⚠ Warning: Binary not found: $binary_path${NC}"
        WARNINGS=$((WARNINGS + 1))
        return
    fi
    
    # Get actual size
    local actual_size=$(stat -c%s "$binary_path" 2>/dev/null || stat -f%z "$binary_path" 2>/dev/null)
    local actual_human=$(format_bytes $actual_size)
    local max_human=$(format_bytes $max_size)
    
    # Calculate percentage
    local percent=$(awk "BEGIN {printf \"%.1f\", ($actual_size/$max_size)*100}")
    local warn_threshold=$(awk "BEGIN {printf \"%.0f\", $max_size*$warn_percent/100}")
    
    # Determine status
    local status="PASS"
    local color=$GREEN
    local symbol="✓"
    
    if [ $actual_size -gt $max_size ]; then
        status="FAIL"
        color=$RED
        symbol="✗"
        FAILURES=$((FAILURES + 1))
    elif [ $actual_size -gt $warn_threshold ]; then
        status="WARN"
        color=$YELLOW
        symbol="⚠"
        WARNINGS=$((WARNINGS + 1))
    else
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
    fi
    
    # Print result
    echo -e "${color}${symbol} ${binary_name}${NC}"
    echo "  Path: $binary_path"
    echo "  Size: $actual_human ($actual_size bytes)"
    echo "  Threshold: $max_human ($max_size bytes)"
    echo "  Usage: ${percent}%"
    echo "  Status: $status"
    echo ""
    
    # Add to results JSON
    RESULTS=$(echo "$RESULTS" | jq --arg name "$binary_name" \
                                    --arg path "$binary_path" \
                                    --argjson size "$actual_size" \
                                    --argjson max "$max_size" \
                                    --arg status "$status" \
                                    --arg percent "$percent" \
                                    --arg desc "$description" \
                                    '. += [{
                                        "name": $name,
                                        "path": $path,
                                        "actual_size": $size,
                                        "max_size": $max,
                                        "percent": $percent,
                                        "status": $status,
                                        "description": $desc
                                    }]')
}

# Read thresholds and check each binary
echo -e "${BLUE}Checking binaries...${NC}"
echo ""

# Check glassbox-cli
if jq -e '.thresholds."glassbox-cli"' "$CONFIG_FILE" > /dev/null; then
    max_size=$(jq -r '.thresholds."glassbox-cli".max_size_bytes' "$CONFIG_FILE")
    warn_percent=$(jq -r '.thresholds."glassbox-cli".warn_threshold_percent' "$CONFIG_FILE")
    description=$(jq -r '.thresholds."glassbox-cli".description' "$CONFIG_FILE")
    check_binary "glassbox-cli" "$TARGET_DIR/glassbox-cli" "$max_size" "$warn_percent" "$description"
fi

# Check watch-demo
if jq -e '.thresholds."watch-demo"' "$CONFIG_FILE" > /dev/null; then
    max_size=$(jq -r '.thresholds."watch-demo".max_size_bytes' "$CONFIG_FILE")
    warn_percent=$(jq -r '.thresholds."watch-demo".warn_threshold_percent' "$CONFIG_FILE")
    description=$(jq -r '.thresholds."watch-demo".description' "$CONFIG_FILE")
    check_binary "watch-demo" "$TARGET_DIR/watch-demo" "$max_size" "$warn_percent" "$description"
fi

# Check library artifact
if jq -e '.thresholds."libglassbox.rlib"' "$CONFIG_FILE" > /dev/null; then
    max_size=$(jq -r '.thresholds."libglassbox.rlib".max_size_bytes' "$CONFIG_FILE")
    warn_percent=$(jq -r '.thresholds."libglassbox.rlib".warn_threshold_percent' "$CONFIG_FILE")
    description=$(jq -r '.thresholds."libglassbox.rlib".description' "$CONFIG_FILE")
    
    # Find the library file (name may vary with Rust version)
    lib_file=$(find "$TARGET_DIR" -name "libglassbox.rlib" -o -name "libglassbox-*.rlib" | head -1)
    if [ -n "$lib_file" ]; then
        check_binary "libglassbox.rlib" "$lib_file" "$max_size" "$warn_percent" "$description"
    fi
fi

# Print summary
echo -e "${BLUE}=== Summary ===${NC}"
echo "Total checks: $TOTAL_CHECKS"
echo -e "${GREEN}Passed: $PASSED_CHECKS${NC}"
echo -e "${YELLOW}Warnings: $WARNINGS${NC}"
echo -e "${RED}Failures: $FAILURES${NC}"
echo ""

# Write results to file
SUMMARY=$(jq -n --argjson results "$RESULTS" \
                --argjson total "$TOTAL_CHECKS" \
                --argjson passed "$PASSED_CHECKS" \
                --argjson warnings "$WARNINGS" \
                --argjson failures "$FAILURES" \
                --arg timestamp "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
                '{
                    "timestamp": $timestamp,
                    "summary": {
                        "total": $total,
                        "passed": $passed,
                        "warnings": $warnings,
                        "failures": $failures
                    },
                    "results": $results
                }')

echo "$SUMMARY" > "$OUTPUT_FILE"
echo "Results written to: $OUTPUT_FILE"
echo ""

# Exit with appropriate code
if [ $FAILURES -gt 0 ]; then
    echo -e "${RED}❌ Binary size check FAILED${NC}"
    exit 1
elif [ $WARNINGS -gt 0 ]; then
    echo -e "${YELLOW}⚠ Binary size check passed with WARNINGS${NC}"
    exit 0
else
    echo -e "${GREEN}✅ Binary size check PASSED${NC}"
    exit 0
fi
