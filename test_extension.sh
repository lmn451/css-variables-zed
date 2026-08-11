#!/bin/bash
# Integration test script for zed-css-variables extension
# This script validates the extension build and structure

set -euo pipefail

echo "🧪 Testing zed-css-variables extension..."

# Color codes for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test 1: Check required files exist
echo -e "\n${YELLOW}Test 1: Checking required files...${NC}"
if [ ! -f "extension.toml" ]; then
    echo -e "${RED}❌ extension.toml not found${NC}"
    exit 1
fi
if [ ! -f "extension.wasm" ]; then
    echo -e "${RED}❌ extension.wasm not found${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Required files present${NC}"

# Test 2: Validate extension.toml structure
echo -e "\n${YELLOW}Test 2: Validating extension.toml...${NC}"
if ! grep -q "schema_version = 1" extension.toml; then
    echo -e "${RED}❌ Invalid schema_version${NC}"
    exit 1
fi
if ! grep -q 'id = "css-variables"' extension.toml; then
    echo -e "${RED}❌ Invalid extension id${NC}"
    exit 1
fi
VERSION_LINE=$(grep -E '^version = "' extension.toml || true)
if [ -z "$VERSION_LINE" ]; then
    echo -e "${RED}❌ Version line missing in extension.toml${NC}"
    exit 1
fi
VERSION=$(echo "$VERSION_LINE" | sed -E 's/.*"([^"]+)".*/\1/')
if [ -z "$VERSION" ]; then
    echo -e "${RED}❌ Failed to parse version from extension.toml${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Version: $VERSION${NC}"
if ! grep -q 'kind = "download_file"' extension.toml; then
    echo -e "${RED}❌ download_file capability not declared${NC}"
    exit 1
fi
if ! grep -q 'host = "github.com"' extension.toml; then
    echo -e "${RED}❌ download_file host not declared${NC}"
    exit 1
fi
if ! grep -q 'path = \["lmn451", "css-lsp-rust", "\*\*"\]' extension.toml; then
    echo -e "${RED}❌ download_file path not declared${NC}"
    exit 1
fi
echo -e "${GREEN}✓ extension.toml valid${NC}"

# Test 3: Verify WASM file is valid
echo -e "\n${YELLOW}Test 3: Checking WASM file...${NC}"
if ! file extension.wasm | grep -q "WebAssembly"; then
    echo -e "${RED}❌ extension.wasm is not a valid WebAssembly file${NC}"
    exit 1
fi
WASM_SIZE=$(stat -f%z extension.wasm 2>/dev/null || stat -c%s extension.wasm 2>/dev/null)
if [ "$WASM_SIZE" -lt 10000 ]; then
    echo -e "${RED}❌ WASM file suspiciously small ($WASM_SIZE bytes)${NC}"
    exit 1
fi
echo -e "${GREEN}✓ WASM file valid (${WASM_SIZE} bytes)${NC}"

# Test 4: Check Rust source for correct version
echo -e "\n${YELLOW}Test 4: Verifying LSP release settings in source...${NC}"
if ! grep -q 'CSS_VARIABLES_RELEASE_REPO' src/lib.rs; then
    echo -e "${RED}❌ Release repo not defined in src/lib.rs${NC}"
    exit 1
fi
if ! grep -q 'latest_github_release' src/lib.rs; then
    echo -e "${RED}❌ latest GitHub release resolution not defined in src/lib.rs${NC}"
    exit 1
fi
if ! grep -q 'build_npm_fallback_command' src/lib.rs; then
    echo -e "${RED}❌ npm fallback not defined in src/lib.rs${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Source code release settings present (latest + npm fallback)${NC}"

# Test 5: Verify example files exist for testing
echo -e "\n${YELLOW}Test 5: Checking example files...${NC}"
if [ ! -f "example/index.html" ] || [ ! -f "example/index.css" ]; then
    echo -e "${RED}❌ Example files missing${NC}"
    exit 1
fi
if ! grep -q "\-\-primary" example/index.css; then
    echo -e "${RED}❌ Example CSS doesn't contain test variables${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Example files present${NC}"

# Test 6: Verify WASM comparator
echo -e "\n${YELLOW}Test 6: Testing WASM comparator...${NC}"
python3 scripts/test_compare_wasm.py
echo -e "${GREEN}✓ WASM comparator tests passed${NC}"

# Test 7: Build test
echo -e "\n${YELLOW}Test 7: Testing build process...${NC}"
./scripts/build_wasm.sh
echo -e "${GREEN}✓ Build successful${NC}"

# Test 8: Verify built WASM matches current
echo -e "\n${YELLOW}Test 8: Verifying WASM is up-to-date...${NC}"
BUILT_WASM="target/wasm32-wasip1/release/zed_css_variables.wasm"
if [ ! -f "$BUILT_WASM" ]; then
    echo -e "${RED}❌ Built WASM file not found: $BUILT_WASM${NC}"
    exit 1
fi

if ! python3 scripts/compare_wasm.py "$BUILT_WASM" extension.wasm; then
    BUILT_SIZE=$(stat -f%z "$BUILT_WASM" 2>/dev/null || stat -c%s "$BUILT_WASM" 2>/dev/null)
    CURRENT_SIZE=$(stat -f%z extension.wasm 2>/dev/null || stat -c%s extension.wasm 2>/dev/null)
    echo -e "${RED}❌ WASM artifact is stale or contains a meaningful build difference${NC}"
    echo -e "${RED}   Built: $BUILT_SIZE bytes${NC}"
    echo -e "${RED}   Current: $CURRENT_SIZE bytes${NC}"
    echo -e "${YELLOW}  Run: cp $BUILT_WASM extension.wasm${NC}"
    exit 1
fi
echo -e "${GREEN}✓ WASM artifact matches the canonicalized pinned stable build${NC}"

echo -e "\n${GREEN}========================================${NC}"
echo -e "${GREEN}✅ All tests passed!${NC}"
echo -e "${GREEN}========================================${NC}"
echo -e "\nExtension is ready for deployment."
echo -e "To test in Zed: Extensions → Install Dev Extension → Select this directory"
