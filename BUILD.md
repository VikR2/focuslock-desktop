# Building FocusLock Desktop App

## Prerequisites

- Docker and Docker Compose (for cross-platform builds)
- Or native Rust toolchain for your platform

## Building with Docker (Recommended for Cross-Platform)

The project includes Docker configurations for building Windows and Linux packages from any platform.

### Build Windows Package

```bash
# Build Windows .exe installer using Docker
docker-compose run --rm windows-builder

# Output will be in: src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/
# Installer copied to: releases/FocusLock_*_x64-setup.exe
```

### Build Linux Package

```bash
# Build Linux .deb package using Docker  
docker-compose run --rm linux-builder

# Output will be in: src-tauri/target/release/bundle/deb/
# Package copied to: releases/focuslock_*_amd64.deb
```

### Build All Platforms

```bash
# Build both Windows and Linux in one command
docker-compose up
```

## Building Natively

### Windows

```bash
# Install Rust
winget install Rustlang.Rust.GNU

# Install Node.js dependencies
npm install

# Build the app
npm run build
cd src-tauri
cargo build --release

# Package will be in: target/release/bundle/nsis/
```

### Linux

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install system dependencies
sudo apt update
sudo apt install -y libwebkit2gtk-4.1-dev \
  build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev

# Install Node.js dependencies
npm install

# Build the app
npm run build
cd src-tauri
cargo build --release

# Package will be in: target/release/bundle/deb/
```

## Project Structure

```
.
├── src-tauri/              # Tauri (Rust) backend
│   ├── src/
│   │   └── main.rs        # Main Rust code, spawns Express server
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── client/                 # React frontend
├── server/                 # Express backend
├── dist/                   # Built backend (bundled with app)
│   └── index.js           # Express server bundle
├── Dockerfile.windows      # Windows build container
├── Dockerfile.linux        # Linux build container
└── docker-compose.yml      # Multi-platform build orchestration
```

## Build Process

1. **Frontend Build** (`vite build`)
   - Compiles React app to `dist/public/`
   
2. **Backend Build** (`esbuild`)
   - Bundles Express server to `dist/index.js`
   
3. **Tauri Build** (`cargo build --release`)
   - Compiles Rust code
   - Bundles frontend assets
   - Includes backend as resource files
   - Creates platform-specific installers

## How Auto-Start Works

The desktop app includes an embedded backend server:

1. **On App Launch:**
   - Rust code checks if Node.js is installed
   - Spawns `node dist/index.js` process
   - Polls `/api/health` endpoint until server responds
   - Shows app window once backend is ready

2. **On App Close:**
   - Rust gracefully kills the Node.js process

3. **Resource Bundling:**
   - `dist/*` files are bundled as Tauri resources
   - Accessible at runtime via `app.path().resource_dir()`

## Troubleshooting

### "Node.js not found" error
The app requires Node.js to be installed on the target system. Make sure Node.js is in the system PATH.

### For truly standalone deployment
To eliminate the Node.js dependency, you can:

1. Use `pkg` to create a standalone executable:
   ```bash
   npm install -g pkg
   pkg dist/index.js --targets node18-win-x64 --output binaries/server.exe
   ```

2. Update `tauri.conf.json` to use the packaged binary:
   ```json
   "externalBin": ["binaries/server"]
   ```

3. Update `src-tauri/src/main.rs` to spawn the binary instead of Node.js

### Build fails with "resource not found"
Ensure you run `npm run build` before building the Tauri app. The backend must be compiled to `dist/index.js` first.

## Release Checklist

- [ ] Update version in `src-tauri/tauri.conf.json`
- [ ] Update version in `package.json`
- [ ] Run `npm run build` to build frontend and backend
- [ ] Build platform packages with Docker
- [ ] Test installers on target platforms
- [ ] Copy packages to `releases/` folder
- [ ] Create release notes
