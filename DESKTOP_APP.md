# FocusLock Desktop App

## Overview

FocusLock is available as a standalone desktop application for Windows and Linux. The desktop app provides native OS integration for detecting and managing applications during focus sessions.

## System Requirements

### Windows
- Windows 10 or later (64-bit)
- **Node.js 18 or later** (required for backend server)
- 100 MB free disk space

### Linux  
- Ubuntu 20.04 or later / equivalent distribution
- **Node.js 18 or later** (required for backend server)
- 100 MB free disk space

## Installation

### Windows
1. Download `FocusLock_x.x.x_x64-setup.exe` from the releases folder
2. Ensure Node.js is installed: https://nodejs.org/
3. Run the installer and follow the prompts
4. Launch FocusLock from the Start Menu or Desktop shortcut

### Linux
1. Download `focuslock_x.x.x_amd64.deb` from the releases folder
2. Ensure Node.js is installed: `sudo apt install nodejs`
3. Install: `sudo dpkg -i focuslock_x.x.x_amd64.deb`
4. Launch: `focuslock` from terminal or app launcher

## How It Works

The desktop app consists of two components:

1. **Frontend (Tauri + React)** - Desktop UI built with Tauri
2. **Backend (Express Server)** - Bundled Node.js server for data persistence

When you launch the app:
- The Rust backend automatically starts the Express server on `localhost:5000`
- A health check ensures the server is ready before showing the UI
- On app close, the server process is gracefully terminated

## Features

### Windows-Specific
- **Registry-based app detection** - Scans Windows Registry for installed applications
- **Process monitoring** - Real-time detection of running processes
- **Native notifications** - Windows toast notifications for session reminders

### Cross-Platform
- Customizable focus timers (Pomodoro, custom durations)
- Application blocking rules
- Favorites bar for quick app access
- Session history and statistics
- Light/Dark theme support

## Building from Source

See `BUILD.md` for instructions on building the desktop app from source.

## Known Limitations

- **Node.js dependency**: The app requires Node.js to be installed on the system since the backend runs as a Node.js process
- **For truly standalone deployment**: Consider using `pkg` or similar tools to bundle Node.js runtime with the backend server

## Future Improvements

- [ ] Bundle Node.js runtime to eliminate external dependency
- [ ] macOS support with native app detection
- [ ] System tray integration
- [ ] Auto-updater for seamless updates
