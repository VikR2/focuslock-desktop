#!/bin/bash
set -e

echo "Building frontend and backend for desktop..."
npm run build

echo "✅ Desktop build complete!"
echo "  - Frontend: dist/public/"
echo "  - Backend: dist/index.js"
