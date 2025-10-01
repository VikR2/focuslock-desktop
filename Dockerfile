FROM rust:1.82-slim

# Install Linux system dependencies for Tauri 2.0
RUN apt-get update && apt-get install -y \
    libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js (for frontend build)
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs

# Install Tauri CLI
RUN cargo install tauri-cli --version ^2.0

# Set working directory
WORKDIR /app

# Copy package files first for better caching
COPY package*.json ./
RUN npm install

# Copy source code
COPY . .

# Note: Build happens via Tauri's beforeBuildCommand during cargo tauri build
# No need to run npm run build here as it will be triggered by Tauri

# Build Tauri app
CMD ["cargo", "tauri", "build"]