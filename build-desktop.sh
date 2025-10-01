#!/bin/bash
set -e
set -o pipefail

echo "Building frontend..."
npx vite build --mode production

echo "Building self-contained backend bundle..."
npx esbuild server/index.ts \
  --platform=node \
  --bundle \
  --format=cjs \
  --outfile=dist/index.js \
  --external:@tauri-apps/* \
  --log-level=warning

echo "✅ Desktop build complete!"
echo "  - Frontend: dist/public/"
echo "  - Backend: dist/index.js (self-contained)"
