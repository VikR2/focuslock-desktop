# FocusLock Build Instructions

## Architecture Overview

FocusLock is available in two deployment modes:

1. **Web App**: Browser-based with Node.js/Express backend and PostgreSQL
2. **Desktop App**: Native Tauri application with Rust backend and SQLite

## Building the Desktop App

### Prerequisites
- Docker installed on your system
- At least 10GB of available disk space

### Build Commands

The build system uses Docker to create cross-platform installers.

#### Linux Build
```bash
./build.sh linux
```

Outputs:
- `.deb` package (Debian/Ubuntu)
- `.rpm` package (Fedora/RHEL)
- `.AppImage` (Universal Linux)

Location: `./releases/linux/`

#### Windows Build
```bash
./build.sh windows
```

Outputs:
- `.exe` NSIS installer

Location: `./releases/windows/`

### Build Process

1. **Frontend Build**: `npx vite build` creates optimized React app in `dist/public/`
2. **Docker Compilation**: Tauri compiles Rust app with embedded frontend
3. **Installer Creation**: Platform-specific installers are generated
4. **Output**: Installers copied to `./releases/` directory

### Testing the Desktop App

#### Enable DevTools
DevTools are now enabled by default in `src-tauri/tauri.conf.json`:
```json
{
  "app": {
    "windows": [{
      "devtools": true
    }]
  }
}
```

#### Run the Desktop App
After building, install the appropriate package for your platform:
- **Linux**: `sudo dpkg -i ./releases/linux/*.deb` or run the AppImage
- **Windows**: Run the `.exe` installer

#### Check Console Logs
Press F12 to open DevTools and verify:
- `[Platform] Detected: Tauri (Desktop Mode)` appears on startup
- `[API] Tauri mode: GET /api/favorites` shows API calls routing through Tauri
- `[API] Invoking: get_favorites` confirms Tauri commands are being called
- No "failed to fetch" errors appear

### Architecture Differences

**Web Mode:**
- Uses HTTP fetch to Node.js/Express backend
- PostgreSQL database
- Session-based authentication
- Console shows: `[Platform] Detected: Web Mode`

**Desktop Mode:**
- Uses Tauri `invoke()` to call Rust commands directly
- SQLite database in app data directory
- No authentication (single-user application)
- Console shows: `[Platform] Detected: Tauri (Desktop Mode)`

### Troubleshooting

#### White Screen / Failed to Fetch
This error occurs when the app runs in desktop mode but tries to make HTTP fetch calls. The fixes include:

1. **Platform Detection**: Async detection with console logging
2. **API Routing**: All endpoints mapped to Tauri commands
3. **Error Handling**: Clear error messages for unmapped endpoints
4. **Retry Logic**: Prevents infinite retries on fetch errors

Check console logs (F12) to see which endpoint is failing.

#### Missing Tauri Commands
If you see errors like `Unhandled Tauri endpoint: GET /api/xxx`, the endpoint needs to be added:

1. Add Rust command in `src-tauri/src/db.rs`
2. Register command in `src-tauri/src/main.rs`
3. Add mapping in `client/src/lib/queryClient.ts`

## Running the Web App (Development)

```bash
npm run dev
```

This starts:
- Express server on port 5000
- Vite dev server with HMR
- PostgreSQL database connection

## Project Structure

```
focuslock/
├── build.sh                    # Build script for desktop apps
├── src-tauri/                  # Rust backend for desktop
│   ├── src/
│   │   ├── main.rs            # Tauri app entry point
│   │   └── db.rs              # SQLite database layer
│   ├── Cargo.toml             # Rust dependencies
│   └── tauri.conf.json        # Tauri configuration
├── client/                     # React frontend
│   └── src/
│       ├── lib/queryClient.ts # Platform detection & API routing
│       └── hooks/             # Data fetching hooks
├── server/                     # Node.js backend (web only)
│   ├── index.ts               # Express server
│   └── routes.ts              # API routes
└── releases/                   # Build outputs
    ├── linux/
    └── windows/
```

## Database Schema

Both SQLite (desktop) and PostgreSQL (web) use the same schema:

- **favorites**: User's favorite applications
- **block_rules**: Application blocking rules
- **sessions**: Focus session history
- **settings**: User preferences

## API Endpoints

All endpoints work in both web and desktop modes:

| Endpoint | Method | Tauri Command |
|----------|--------|---------------|
| `/api/favorites` | GET | `get_favorites` |
| `/api/favorites` | POST | `create_favorite` |
| `/api/favorites/:id` | DELETE | `delete_favorite` |
| `/api/block-rules` | GET | `get_block_rules` |
| `/api/block-rules` | POST | `create_block_rule` |
| `/api/block-rules/:id` | PATCH | `update_block_rule` |
| `/api/block-rules/:id` | DELETE | `delete_block_rule` |
| `/api/sessions` | GET | `get_sessions` |
| `/api/sessions` | POST | `create_session` |
| `/api/sessions/:id` | PATCH | `update_session` |
| `/api/settings` | GET | `get_settings` |
| `/api/settings` | POST | `upsert_setting` |
