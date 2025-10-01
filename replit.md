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
- **Runtime**: Node.js with Express.js server
- **Language**: TypeScript throughout the stack
- **API Design**: RESTful endpoints for session management, favorites, block rules, and settings
- **Request Handling**: Express middleware for JSON parsing, logging, and error handling
- **Development**: Vite integration for hot module replacement in development

### Data Storage Solutions
- **Database**: PostgreSQL with Drizzle ORM
- **Connection**: Neon Database serverless connection (@neondatabase/serverless)
- **Schema Management**: Drizzle-kit for migrations and schema management
- **In-Memory Fallback**: MemStorage class implements the same interface for development/testing
- **Data Models**: Favorites (pinned apps), BlockRules (app blocking configuration), Sessions (focus session tracking), Settings (user preferences)

### Authentication and Authorization
- Currently using basic session-based approach with connect-pg-simple for PostgreSQL session storage
- No complex authentication system implemented - appears to be single-user focused application

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

### Tauri Integration (Added: September 2025)
- **Desktop Framework**: Tauri v2.4 for cross-platform desktop apps
- **Rust Backend**: Native OS integration for Windows app detection and process monitoring
- **Embedded Server**: Express backend auto-starts with desktop app, bundled as Tauri resources
- **Build System**: Docker-based cross-compilation for Windows (cargo-xwin) and Linux packages

### Desktop-Specific Features
- **Windows Integration**:
  - Registry-based installed app detection using `winreg` crate
  - Real-time process monitoring with `sysinfo` crate  
  - Native Windows notifications
  - NSIS installer packaging

- **Backend Auto-Start**:
  - Rust spawns Node.js process on app launch (`node dist/index.js`)
  - Health check polling (`/api/health`) ensures backend readiness
  - Graceful process cleanup on app exit
  - Requires Node.js installed on target system

### Build Configuration (Updated: October 2025)
- **Simplified Build System**: Single `build.sh` script for both platforms
- **Build Flow**:
  1. Local build: `npm run build` creates frontend (`dist/public/`) and backend (`dist/index.js`)
  2. Docker compilation: Tauri compiles Rust wrapper around pre-built files
  3. Output: Installers copied to `./releases/windows/` or `./releases/linux/`
- **Output Formats**: 
  - Windows: `.exe` NSIS installer (via cargo-xwin cross-compilation)
  - Linux: `.deb`, `.rpm`, and `.AppImage` packages

### Key Files
- `build.sh`: Single build script for all platforms (`./build.sh linux` or `./build.sh windows`)
- `src-tauri/src/main.rs`: Rust main with backend spawning and health checks
- `src-tauri/tauri.conf.json`: Tauri configuration with resource bundling
- `src-tauri/Cargo.toml`: Rust dependencies (winreg, sysinfo, reqwest, tokio)
- `Dockerfile`: Linux build container (Rust + Tauri only)
- `Dockerfile.windows`: Windows cross-compilation container (Rust + Tauri + cargo-xwin)
- `DESKTOP_APP.md`: User installation and usage guide