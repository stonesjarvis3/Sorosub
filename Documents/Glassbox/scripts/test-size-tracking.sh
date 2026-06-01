#!/bin/bash
# Test script for binary size tracking functionality
# This script validates that size tracking works correctly

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Testing Binary Size Tracking ===${NC}"
echo ""

TESTS_PASSED=0
TESTS_FAILED=0

# Function to run a test
run_test() {
    local test_name=$1
    local test_command=$2
    local expected_exit_code=${3:-0}
    
    echo -e "${BLUE}Test: $test_name${NC}"
    
    if eval "$test_command"; then
        actual_exit=$?
    else
        actual_exit=$?
    fi
    
    if [ $actual_exit -eq $expected_exit_code ]; then
        echo -e "${GREEN}✓ PASSED${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}✗ FAILED (expected exit code $expected_exit_code, got $actual_exit)${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
    echo ""
}

# Test 1: Check if configuration file exists and is valid JSON
run_test "Configuration file exists and is valid JSON" \
    "jq empty size-thresholds.json"

# Test 2: Check if configuration has required fields
run_test "Configuration has required fields" \
    "jq -e '.version and .thresholds' size-thresholds.json > /dev/null"

# Test 3: Check if size check script exists and is executable
run_test "Size check script exists and is executable" \
    "test -x scripts/check-binary-sizes.sh"

# Test 4: Check if baseline measurement script exists and is executable
run_test "Baseline measurement script exists and is executable" \
    "test -x scripts/measure-baseline-sizes.sh"

# Test 5: Check if GitHub Actions workflow exists
run_test "GitHub Actions workflow exists" \
    "test -f .github/workflows/size-check.yml"

# Test 6: Validate workflow YAML syntax
if command -v yamllint &> /dev/null; then
    run_test "GitHub Actions workflow has valid YAML syntax" \
        "yamllint -d relaxed .github/workflows/size-check.yml"
else
    echo -e "${YELLOW}⚠ Skipping YAML validation (yamllint not installed)${NC}"
    echo ""
fi

# Test 7: Check if jq is available (required dependency)
run_test "jq is installed (required dependency)" \
    "command -v jq > /dev/null"

# Test 8: Verify configuration has thresholds for expected binaries
run_test "Configuration has threshold for glassbox-cli" \
    "jq -e '.thresholds.\"glassbox-cli\"' size-thresholds.json > /dev/null"

run_test "Configuration has threshold for watch-demo" \
    "jq -e '.thresholds.\"watch-demo\"' size-thresholds.json > /dev/null"

# Test 9: Verify threshold values are reasonable
run_test "Thresholds have positive max_size_bytes" \
    "jq -e '.thresholds | to_entries[] | select(.value.max_size_bytes > 0)' size-thresholds.json > /dev/null"

run_test "Thresholds have valid warn_threshold_percent (0-100)" \
    "jq -e '.thresholds | to_entries[] | select(.value.warn_threshold_percent >= 0 and .value.warn_threshold_percent <= 100)' size-thresholds.json > /dev/null"

# Test 10: Test size check script with missing target directory (should fail gracefully)
echo -e "${BLUE}Test: Size check script handles missing target directory${NC}"
if TARGET_DIR="/nonexistent" ./scripts/check-binary-sizes.sh 2>&1 | grep -q "Error.*not found"; then
    echo -e "${GREEN}✓ PASSED${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAILED${NC}"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test 11: If binaries exist, run actual size check
if [ -d "target/release" ] && [ -f "target/release/glassbox-cli" ]; then
    echo -e "${BLUE}Test: Size check script runs successfully with built binaries${NC}"
    if ./scripts/check-binary-sizes.sh > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PASSED${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        # It's okay if it fails due to size, as long as it runs
        if [ -f "size-report.json" ]; then
            echo -e "${GREEN}✓ PASSED (script executed and generated report)${NC}"
            TESTS_PASSED=$((TESTS_PASSED + 1))
        else
            echo -e "${RED}✗ FAILED${NC}"
            TESTS_FAILED=$((TESTS_FAILED + 1))
        fi
    fi
    echo ""
    
    # Test 12: Verify size report is generated
    run_test "Size report JSON is generated" \
        "test -f size-report.json"
    
    # Test 13: Verify size report has valid structure
    run_test "Size report has valid structure" \
        "jq -e '.timestamp and .summary and .results' size-report.json > /dev/null"
else
    echo -e "${YELLOW}⚠ Skipping runtime tests (binaries not built)${NC}"
    echo "  Run 'cargo build --release' to enable full testing"
    echo ""
fi

# Print summary
echo -e "${BLUE}=== Test Summary ===${NC}"
echo "Tests passed: ${GREEN}$TESTS_PASSED${NC}"
echo "Tests failed: ${RED}$TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed${NC}"
    exit 1
fi
