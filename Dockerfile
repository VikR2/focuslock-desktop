FROM rust:latest

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

# Install Tauri CLI (use locked to avoid dependency resolution issues)
RUN cargo install tauri-cli --locked

# Working directory will be set by docker run command
WORKDIR /app
