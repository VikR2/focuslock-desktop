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

# Install Tauri CLI
RUN cargo install tauri-cli --version ^2.0

# Working directory will be set by docker run command
WORKDIR /app
