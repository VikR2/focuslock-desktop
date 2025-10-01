#!/bin/bash
set -e

# Change to the script's directory (repo root)
cd "$(dirname "$0")"

echo "Building frontend and backend for desktop..."
npm run build

echo "✅ Desktop build complete!"
echo "  - Frontend: dist/public/"
echo "  - Backend: dist/index.js"
