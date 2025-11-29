#!/usr/bin/env bash

# Code Coverage Script for gridskipper
# This script generates code coverage reports using LLVM coverage tools

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROFILE_FILE="gridskipper.profdata"
COVERAGE_DIR="coverage"
RUSTFLAGS_COV="-C instrument-coverage"
LLVM_PROFILE_FILE_PATTERN="gridskipper-%p-%m.profraw"

# Get the project root directory (parent of scripts directory)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo -e "${BLUE}🔍 Generating code coverage for gridskipper...${NC}"

# Clean up previous coverage data
echo -e "${YELLOW}🧹 Cleaning up previous coverage data...${NC}"
rm -f *.profraw *.profdata
rm -rf "$COVERAGE_DIR"

# Build and run tests with coverage instrumentation
echo -e "${YELLOW}🔨 Building and running tests with coverage instrumentation...${NC}"
RUSTFLAGS="$RUSTFLAGS_COV" LLVM_PROFILE_FILE="$LLVM_PROFILE_FILE_PATTERN" cargo test

# Check if profile files were generated
if ! ls gridskipper-*.profraw 1> /dev/null 2>&1; then
    echo -e "${RED}❌ Error: No profile files generated. Make sure tests ran successfully.${NC}"
    exit 1
fi

# Find LLVM tools
RUSTUP_TOOLCHAIN=$(rustc --version --verbose | grep "host:" | cut -d' ' -f2)
RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
LLVM_TOOLS_DIR="$RUSTUP_HOME/toolchains/stable-$RUSTUP_TOOLCHAIN/lib/rustlib/$RUSTUP_TOOLCHAIN/bin"

if [[ ! -d "$LLVM_TOOLS_DIR" ]]; then
    echo -e "${RED}❌ Error: Could not find LLVM tools directory: $LLVM_TOOLS_DIR${NC}"
    echo -e "${YELLOW}💡 Make sure you have the Rust toolchain properly installed.${NC}"
    exit 1
fi

LLVM_PROFDATA="$LLVM_TOOLS_DIR/llvm-profdata"
LLVM_COV="$LLVM_TOOLS_DIR/llvm-cov"

# Merge profile data
echo -e "${YELLOW}📊 Merging profile data...${NC}"
"$LLVM_PROFDATA" merge -sparse gridskipper-*.profraw -o "$PROFILE_FILE"

# Find the test binary with coverage data
echo -e "${YELLOW}🔍 Finding test binary with coverage data...${NC}"
TEST_BINARY=""
BEST_COVERAGE=0

for binary in $(find target/debug/deps -name "gridskipper-*" -type f -perm +111 | grep -E "gridskipper-[a-f0-9]+$"); do
    # Test if this binary has coverage data and get coverage percentage
    if coverage_output=$("$LLVM_COV" report --instr-profile="$PROFILE_FILE" "$binary" --sources src 2>/dev/null); then
        # Extract the overall coverage percentage from the last line (4th field from end)
        coverage_percent=$(echo "$coverage_output" | tail -1 | awk '{print $(NF-3)}' | sed 's/%//' | cut -d'.' -f1)
        if [[ "$coverage_percent" =~ ^[0-9]+$ ]] && [[ "$coverage_percent" -gt "$BEST_COVERAGE" ]]; then
            BEST_COVERAGE="$coverage_percent"
            TEST_BINARY="$binary"
        elif [[ -z "$TEST_BINARY" ]]; then
            # If no binary found yet, use this one as fallback
            TEST_BINARY="$binary"
        fi
    fi
done

if [[ -z "$TEST_BINARY" ]]; then
    echo -e "${RED}❌ Error: Could not find test binary with coverage data in target/debug/deps/${NC}"
    echo -e "${YELLOW}💡 Available binaries:${NC}"
    find target/debug/deps -name "gridskipper-*" -type f -perm +111 | grep -E "gridskipper-[a-f0-9]+$" | head -5
    exit 1
fi

echo -e "${YELLOW}🎯 Using test binary: $TEST_BINARY${NC}"

# Generate coverage reports
echo -e "${YELLOW}📈 Generating HTML coverage report...${NC}"
"$LLVM_COV" show \
    --format=html \
    --instr-profile="$PROFILE_FILE" \
    --ignore-filename-regex='/.cargo/registry' \
    "$TEST_BINARY" \
    --sources src \
    --output-dir="$COVERAGE_DIR"

# Generate console summary
echo -e "${YELLOW}📋 Generating coverage summary...${NC}"
echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}        COVERAGE SUMMARY        ${NC}"
echo -e "${BLUE}================================${NC}"

"$LLVM_COV" report \
    --instr-profile="$PROFILE_FILE" \
    --ignore-filename-regex='/.cargo/registry' \
    "$TEST_BINARY" \
    --sources src

echo -e "${BLUE}================================${NC}"

# Clean up profile raw files (keep .profdata for potential reuse)
echo -e "${YELLOW}🧹 Cleaning up temporary profile files...${NC}"
rm -f gridskipper-*.profraw

echo -e "${GREEN}✅ Coverage report generated successfully!${NC}"
echo -e "${GREEN}📁 HTML report available at: ${COVERAGE_DIR}/index.html${NC}"
echo -e "${GREEN}🌐 Open in browser with: open ${COVERAGE_DIR}/index.html${NC}"
echo ""
echo -e "${BLUE}💡 Tips:${NC}"
echo -e "${BLUE}  • The HTML report shows line-by-line coverage details${NC}"
echo -e "${BLUE}  • Red lines indicate uncovered code${NC}"
echo -e "${BLUE}  • Green lines indicate covered code${NC}"
echo -e "${BLUE}  • Run this script anytime to refresh coverage data${NC}"
