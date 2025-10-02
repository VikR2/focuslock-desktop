# FocusLock

## Overview

FocusLock is a cross-platform productivity application available as both a web app and desktop application. It helps users block distracting applications during timed focus sessions through customizable timers and application management. The system provides features for managing favorite applications, creating block rules for apps, running timed focus sessions, and configuring user preferences. The application follows Microsoft Fluent Design principles to maintain consistency with Windows productivity tools.

**Deployment Options:**
- **Web App**: Browser-based, accessible via Replit deployment
- **Desktop App**: Native Windows/Linux application with OS integration (Tauri + React + Rust)

## User Preferences

Preferred communication style: Simple, everyday language.

## System Architecture

### Frontend Architecture
- **Framework**: React with TypeScript, using Vite as the build tool
- **UI Library**: Radix UI components with shadcn/ui design system
- **Styling**: Tailwind CSS with custom design tokens following Microsoft Fluent Design
- **State Management**: TanStack React Query for server state, local component state for UI interactions
- **Routing**: Wouter for lightweight client-side routing
- **Component Structure**: Modular components for SessionTimer, SessionPanel, FavoritesBar, AppSearch, RulesTable, and SettingsPanel

### Backend Architecture

**Web App:**
- **Runtime**: Node.js with Express.js server
- **Language**: TypeScript throughout the stack
- **API Design**: RESTful endpoints for session management, favorites, block rules, and settings
- **Request Handling**: Express middleware for JSON parsing, logging, and error handling
- **Development**: Vite integration for hot module replacement in development

**Desktop App (Refactored: October 2025):**
- **Runtime**: Rust-only backend (no Node.js dependency)
- **Database**: SQLite with rusqlite crate
- **API Layer**: Tauri commands invoked directly from frontend
- **Architecture**: Single-process application - frontend and backend run in same process
- **Serialization**: All Rust structs use `#[serde(rename_all = "camelCase")]` to match JavaScript naming conventions

### Data Storage Solutions

**Web App:**
- **Database**: PostgreSQL with Drizzle ORM
- **Connection**: Neon Database serverless connection (@neondatabase/serverless)
- **Schema Management**: Drizzle-kit for migrations and schema management
- **In-Memory Fallback**: MemStorage class implements the same interface for development/testing

**Desktop App:**
- **Database**: SQLite file stored in app data directory
- **Schema**: SQLite tables for favorites, block_rules, sessions, and settings
- **Access**: Direct Rust queries via rusqlite, no ORM
- **Data Models**: Same structure as web app (Favorites, BlockRules, Sessions, Settings)

### Authentication and Authorization
- **Web App**: Basic session-based approach with connect-pg-simple for PostgreSQL session storage
- **Desktop App**: No authentication needed (single-user, local application)

### Design System Integration
- **Theme System**: Light/dark mode support with CSS custom properties
- **Typography**: Segoe UI Variable font family with system fallbacks
- **Color Palette**: Neutral-based theme with productivity-focused blue accents
- **Component Consistency**: shadcn/ui components customized to match Fluent Design principles
- **Responsive Design**: Mobile-first approach with sidebar navigation

## External Dependencies

### Database Services
- **Neon Database**: Serverless PostgreSQL hosting (@neondatabase/serverless v0.10.4)
- **Drizzle ORM**: Database toolkit and ORM (drizzle-orm v0.39.1)
- **Session Storage**: PostgreSQL session store (connect-pg-simple v10.0.0)

### UI and Design Libraries
- **Radix UI**: Comprehensive set of unstyled, accessible UI primitives
- **Tailwind CSS**: Utility-first CSS framework with custom configuration
- **shadcn/ui**: Pre-built component library built on Radix UI
- **Lucide React**: Icon library for consistent iconography
- **Class Variance Authority**: Utility for creating variant-based component APIs

### Development and Build Tools
- **Vite**: Fast build tool and dev server with React plugin
- **TypeScript**: Type safety across frontend and backend
- **ESBuild**: Fast JavaScript bundler for production builds
- **PostCSS**: CSS processing with Tailwind and Autoprefixer

### State Management and Data Fetching
- **TanStack React Query**: Server state management and caching (v5.60.5)
- **React Hook Form**: Form state management with Zod validation
- **Zod**: Runtime type validation and schema validation

### Utility Libraries
- **date-fns**: Date manipulation and formatting (v3.6.0)
- **clsx**: Conditional className utility
- **nanoid**: Unique ID generation
- **wouter**: Lightweight client-side routing

## Desktop Application Architecture

### Tauri Integration (Refactored: October 2025)
- **Desktop Framework**: Tauri v2.4 for cross-platform desktop apps
- **Rust Backend**: Native Rust backend with SQLite database (no Node.js dependency)
- **Architecture**: Single-process application - all backend logic runs in Rust
- **Build System**: Docker-based cross-compilation for Windows (cargo-xwin) and Linux packages

### Desktop-Specific Features
- **Windows Integration**:
  - Registry-based installed app detection using `winreg` crate
  - Real-time process monitoring with `sysinfo` crate  
  - Native Windows notifications
  - NSIS installer packaging

- **Data Persistence**:
  - SQLite database stored in app data directory
  - rusqlite crate for database operations
  - Tauri commands expose CRUD operations to frontend
  - No external dependencies - fully self-contained

- **Frontend-Backend Communication**:
  - Frontend uses Tauri `invoke()` API to call Rust functions
  - Automatic detection: uses Tauri commands in desktop, HTTP fetch in web
  - Type-safe communication with serialization/deserialization

### Build Configuration (Simplified: October 2025)
- **Streamlined Build System**: Single `build.sh` script for both platforms
- **Build Flow**:
  1. Frontend build: `npx vite build` creates `dist/public/`
  2. Docker compilation: Tauri compiles Rust app with embedded frontend
  3. Output: Installers copied to `./releases/windows/` or `./releases/linux/`
- **Output Formats**: 
  - Windows: `.exe` NSIS installer (via cargo-xwin cross-compilation)
  - Linux: `.deb`, `.rpm`, and `.AppImage` packages
- **No Node.js Required**: Desktop app is fully self-contained, no runtime dependencies

### Key Files
- `build.sh`: Single build script for all platforms (`./build.sh linux` or `./build.sh windows`)
- `src-tauri/src/main.rs`: Rust main with database initialization and Tauri commands
- `src-tauri/src/db.rs`: SQLite database layer with all CRUD operations
- `src-tauri/tauri.conf.json`: Tauri configuration (frontend bundling only)
- `src-tauri/Cargo.toml`: Rust dependencies (rusqlite, uuid, winreg, sysinfo)
- `client/src/lib/queryClient.ts`: Frontend API client with Tauri invoke() integration
- `Dockerfile`: Linux build container (Rust + Tauri only)
- `Dockerfile.windows`: Windows cross-compilation container (Rust + Tauri + cargo-xwin)
- `DESKTOP_APP.md`: User installation and usage guide