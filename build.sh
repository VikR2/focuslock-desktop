#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
PLATFORM=${1:-"linux"}

echo -e "${BLUE}FocusLock Desktop Build Script${NC}"
echo "================================"
echo ""

# Step 1: Build frontend and backend locally
echo -e "${BLUE}Step 1: Building frontend and backend...${NC}"
npm run build

if [ ! -f "dist/index.js" ] || [ ! -d "dist/public" ]; then
    echo -e "${RED}Error: Build failed. Missing dist/index.js or dist/public${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Frontend and backend built successfully${NC}"
echo ""

# Step 2: Build desktop app with Docker
if [ "$PLATFORM" == "windows" ]; then
    echo -e "${BLUE}Step 2: Building Windows installer with Docker...${NC}"
    
    # Build Docker image
    docker build -f Dockerfile.windows -t focuslock-windows-builder .
    
    # Run Tauri build in container (no volume over target so outputs reach host)
    # Use MSYS_NO_PATHCONV to prevent Git Bash path conversion on Windows
    # Download Windows SDK and configure linker, then build with cargo-xwin
    # Note: NSIS bundler is automatically used for Windows targets, no --bundles flag needed
    MSYS_NO_PATHCONV=1 docker run --rm \
        -v "$(pwd):/app" \
        -w /app \
        -e XWIN_ACCEPT_LICENSE=1 \
        focuslock-windows-builder \
        sh -c '
            # Download Windows SDK to /xwin
            xwin --accept-license splat --output /xwin
            
            # Create Cargo config for xwin linker
            mkdir -p .cargo
            cat > .cargo/config.toml << EOF
[target.x86_64-pc-windows-msvc]
linker = "lld"
rustflags = [
    "-Lnative=/xwin/crt/lib/x86_64",
    "-Lnative=/xwin/sdk/lib/um/x86_64", 
    "-Lnative=/xwin/sdk/lib/ucrt/x86_64"
]
EOF
            
            # Build with cargo-xwin
            cd src-tauri && cargo xwin build --release --target x86_64-pc-windows-msvc
            
            # Package with Tauri
            cargo tauri build --ci --target x86_64-pc-windows-msvc
        '
    
    # Copy outputs
    mkdir -p releases/windows
    cp -r src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/*.exe releases/windows/ 2>/dev/null || true
    
    echo -e "${GREEN}✓ Windows installer built successfully${NC}"
    echo -e "Output: ${GREEN}releases/windows/*.exe${NC}"

elif [ "$PLATFORM" == "linux" ]; then
    echo -e "${BLUE}Step 2: Building Linux packages with Docker...${NC}"
    
    # Build Docker image
    docker build -f Dockerfile -t focuslock-linux-builder .
    
    # Run Tauri build in container (no volume over target so outputs reach host)
    # Use MSYS_NO_PATHCONV to prevent Git Bash path conversion on Windows
    MSYS_NO_PATHCONV=1 docker run --rm \
        -v "$(pwd):/app" \
        -w /app/src-tauri \
        focuslock-linux-builder \
        cargo tauri build
    
    # Copy outputs
    mkdir -p releases/linux
    cp -r src-tauri/target/release/bundle/deb/*.deb releases/linux/ 2>/dev/null || true
    cp -r src-tauri/target/release/bundle/rpm/*.rpm releases/linux/ 2>/dev/null || true
    cp -r src-tauri/target/release/bundle/appimage/*.AppImage releases/linux/ 2>/dev/null || true
    
    echo -e "${GREEN}✓ Linux packages built successfully${NC}"
    echo -e "Output: ${GREEN}releases/linux/${NC}"

else
    echo -e "${RED}Error: Unknown platform '$PLATFORM'${NC}"
    echo "Usage: ./build.sh [linux|windows]"
    exit 1
fi

echo ""
echo -e "${GREEN}Build complete!${NC}"
