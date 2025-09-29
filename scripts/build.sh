#!/bin/bash

# FocusLock Desktop Build Script

echo "🚀 Building FocusLock Desktop App..."

# Check if Docker is available
if command -v docker >/dev/null 2>&1; then
    echo "📦 Using Docker for containerized build..."
    
    # Determine build target
    case "${1:-all}" in
        "linux")
            echo "🐧 Building Linux packages..."
            docker-compose up tauri-build-linux
            echo "✅ Linux build complete! Check src-tauri/target/release/bundle/"
            ;;
        "windows")
            echo "🪟 Building Windows packages..."
            docker-compose up tauri-build-windows
            echo "✅ Windows build complete! Check src-tauri/target/x86_64-pc-windows-msvc/release/bundle/"
            ;;
        "all"|*)
            echo "🌍 Building for all platforms..."
            docker-compose up tauri-build-linux tauri-build-windows
            echo "✅ All builds complete!"
            echo "   Linux: src-tauri/target/release/bundle/"
            echo "   Windows: src-tauri/target/x86_64-pc-windows-msvc/release/bundle/"
            ;;
    esac
    
else
    echo "❌ Docker not found. Please install Docker or build locally."
    echo "Local build instructions: https://github.com/VikR2/focuslock-desktop#local-building"
    exit 1
fi