#!/bin/bash

# FocusLock Desktop Build Script

echo "🚀 Building FocusLock Desktop App..."

# Check if Docker is available
if command -v docker >/dev/null 2>&1; then
    echo "📦 Using Docker for containerized build..."
    
    # Build using Docker Compose
    docker-compose up tauri-build
    
    echo "✅ Build complete! Check src-tauri/target/release/bundle/ for distributable files"
    
else
    echo "❌ Docker not found. Please install Docker or build locally."
    echo "Local build instructions: https://github.com/VikR2/focuslock-desktop#local-building"
    exit 1
fi