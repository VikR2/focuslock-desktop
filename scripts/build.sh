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
            echo ""
            echo "✅ Linux build complete!"
            echo "📦 Packages available in: releases/linux/"
            ls -lh releases/linux/ 2>/dev/null || true
            ;;
        "windows")
            echo "🪟 Building Windows packages..."
            docker-compose up tauri-build-windows
            echo ""
            echo "✅ Windows build complete!"
            echo "📦 Packages available in: releases/windows/"
            ls -lh releases/windows/ 2>/dev/null || true
            ;;
        "all"|*)
            echo "🌍 Building for all platforms..."
            docker-compose up tauri-build-linux tauri-build-windows
            echo ""
            echo "✅ All builds complete!"
            echo ""
            echo "📦 Your packages are ready in the releases/ folder:"
            echo "   Linux:   releases/linux/"
            echo "   Windows: releases/windows/"
            echo ""
            ls -lh releases/linux/ 2>/dev/null || true
            ls -lh releases/windows/ 2>/dev/null || true
            ;;
    esac
    
else
    echo "❌ Docker not found. Please install Docker or build locally."
    echo "Local build instructions: https://github.com/VikR2/focuslock-desktop#local-building"
    exit 1
fi