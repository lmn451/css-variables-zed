#!/bin/bash
# Docker-based end-to-end test for zed-css-variables extension

set -e

echo "🐳 Building test Docker image..."
docker build -f Dockerfile.test -t zed-css-variables-test .

echo ""
echo "🧪 Running extension tests in clean Docker environment..."
docker run --rm zed-css-variables-test

echo ""
echo "✅ Docker test completed successfully!"
echo ""
echo "The extension has been validated in a fresh Ubuntu environment."
echo "This confirms it will work on new systems with just Zed installed."
