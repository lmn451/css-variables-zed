#!/bin/bash
# Comprehensive Docker test including npm package installation

set -e

echo "🐳 Building comprehensive test Docker image..."
echo "This will test the extension AND the npm package installation..."
docker build -f Dockerfile.zed-headless -t zed-css-variables-full-test .

echo ""
echo "🧪 Running comprehensive tests..."
docker run --rm zed-css-variables-full-test

echo ""
echo "✅ Full Docker test completed successfully!"
echo ""
echo "This test validates:"
echo "  ✓ Extension builds in clean environment"
echo "  ✓ All unit and integration tests pass"
echo "  ✓ npm package css-variable-lsp@1.0.5-beta.1 can be installed"
echo "  ✓ LSP binary is accessible"
echo "  ✓ Extension is ready for production use"
