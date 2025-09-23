# FocusLock Web Prototype

## Overview

FocusLock is a web-based focus session manager designed as a productivity application. It helps users block distracting applications during timed focus sessions through customizable timers and application management. The system provides features for managing favorite applications, creating block rules for apps, running timed focus sessions, and configuring user preferences. The application follows Microsoft Fluent Design principles to maintain consistency with Windows productivity tools.

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