#!/bin/bash
# Simulates a clean installation without Docker
# This test validates what happens on a fresh system

set -euo pipefail

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${YELLOW}🧪 Simulating clean installation test...${NC}\n"

# Create temporary test directory
mkdir -p .context
TEST_DIR=$(mktemp -d "$PWD/.context/clean-install.XXXXXX")
cleanup() {
    if [ "${KEEP_TEST_DIR:-0}" != "1" ]; then
        rm -rf "$TEST_DIR"
    fi
}
trap cleanup EXIT

echo -e "${YELLOW}Step 1: Copying extension files to test directory...${NC}"
cp extension.toml extension.wasm "$TEST_DIR/"
echo -e "${GREEN}✓ Files copied${NC}\n"

echo -e "${YELLOW}Step 2: Verifying extension structure...${NC}"
cd "$TEST_DIR"
if [ ! -f "extension.toml" ] || [ ! -f "extension.wasm" ]; then
    echo -e "${RED}❌ Required files missing${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Extension structure valid${NC}\n"

echo -e "${YELLOW}Step 3: Verifying download capability is declared...${NC}"
if grep -q 'kind = "download_file"' extension.toml && \
   grep -q 'host = "github.com"' extension.toml; then
    echo -e "${GREEN}✓ download_file capability declared${NC}"
else
    echo -e "${RED}❌ download_file capability missing${NC}"
    exit 1
fi

echo -e "\n${YELLOW}Step 4: Testing npm fallback package (css-variable-lsp@latest)...${NC}"
if command -v npm >/dev/null 2>&1; then
    # Create package.json to avoid npm installing in parent directory
    echo '{"name":"test","version":"1.0.0"}' > package.json
    npm install css-variable-lsp@latest --no-save 2>&1 | tail -5
    
    # The binary path is a symlink to the actual server.js file
    if [ -L "node_modules/.bin/css-variable-lsp" ] || [ -f "node_modules/.bin/css-variable-lsp" ]; then
        echo -e "${GREEN}✓ npm fallback LSP binary installed successfully${NC}"
    else
        echo -e "${RED}❌ npm fallback LSP binary not found${NC}"
        exit 1
    fi
    # The extension invokes this JavaScript entrypoint directly on every platform.
    test -f node_modules/css-variable-lsp/out/server.js
    node --check node_modules/css-variable-lsp/out/server.js
    node -p '"Verified css-variable-lsp@" + require("./node_modules/css-variable-lsp/package.json").version'
else
    echo -e "${YELLOW}⚠ npm not available, skipping fallback test${NC}"
fi

cd ../..

echo -e "\n${YELLOW}Step 5: Cleanup...${NC}"
if [ "${KEEP_TEST_DIR:-0}" != "1" ]; then
    rm -rf "$TEST_DIR"
    echo -e "${GREEN}✓ Test directory cleaned up${NC}\n"
else
    echo -e "${YELLOW}⚠ Test directory preserved: $TEST_DIR${NC}\n"
fi

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}✅ Clean installation test PASSED!${NC}"
echo -e "${GREEN}========================================${NC}\n"

echo -e "This test confirms:"
echo -e "  ✓ Extension files are properly structured"
echo -e "  ✓ download_file capability is declared"
echo -e "  ✓ Download capability is present (live Zed download not exercised)"
echo -e "  ✓ npm fallback package is available if needed"
