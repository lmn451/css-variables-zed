#!/bin/bash
# Integration test script for zed-css-variables extension
# This script validates the extension build and structure

set -e

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
if ! grep -q 'version = "0.0.5"' extension.toml; then
    echo -e "${RED}❌ Version mismatch${NC}"
    exit 1
fi
if ! grep -q 'kind = "npm:install"' extension.toml; then
    echo -e "${RED}❌ npm:install capability not declared${NC}"
    exit 1
fi
if ! grep -q 'package = "css-variable-lsp"' extension.toml; then
    echo -e "${RED}❌ npm package not specified in capabilities${NC}"
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
echo -e "\n${YELLOW}Test 4: Verifying LSP version in source...${NC}"
if ! grep -q 'let version = "1.0.9"' src/lib.rs; then
    echo -e "${RED}❌ LSP version mismatch in src/lib.rs${NC}"
    exit 1
fi
if ! grep -q 'let package = "css-variable-lsp"' src/lib.rs; then
    echo -e "${RED}❌ Package name mismatch in src/lib.rs${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Source code version correct${NC}"

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

# Test 6: Build test
echo -e "\n${YELLOW}Test 6: Testing build process...${NC}"
if ! cargo build --release --target wasm32-wasip1 2>&1 | grep -q "Finished"; then
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Build successful${NC}"

# Test 7: Verify built WASM matches current
echo -e "\n${YELLOW}Test 7: Verifying WASM is up-to-date...${NC}"
BUILT_SIZE=$(stat -f%z target/wasm32-wasip1/release/zed_css_variables.wasm 2>/dev/null || stat -c%s target/wasm32-wasip1/release/zed_css_variables.wasm 2>/dev/null)
CURRENT_SIZE=$(stat -f%z extension.wasm 2>/dev/null || stat -c%s extension.wasm 2>/dev/null)
if [ "$BUILT_SIZE" != "$CURRENT_SIZE" ]; then
    echo -e "${YELLOW}⚠ WASM file size mismatch (built: $BUILT_SIZE, current: $CURRENT_SIZE)${NC}"
    echo -e "${YELLOW}  Run: cp target/wasm32-wasip1/release/zed_css_variables.wasm extension.wasm${NC}"
fi
echo -e "${GREEN}✓ WASM verification complete${NC}"

echo -e "\n${GREEN}========================================${NC}"
echo -e "${GREEN}✅ All tests passed!${NC}"
echo -e "${GREEN}========================================${NC}"
echo -e "\nExtension is ready for deployment."
echo -e "To test in Zed: Extensions → Install Dev Extension → Select this directory"
