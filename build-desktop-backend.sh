#!/bin/bash
# Build backend with all dependencies bundled for desktop app

echo "Building standalone backend for desktop..."
npx esbuild server/index.ts \
  --platform=node \
  --bundle \
  --format=esm \
  --outdir=dist \
  --packages=bundle \
  --external:@neondatabase/serverless \
  --external:pg-native

echo "✓ Standalone backend built successfully"
