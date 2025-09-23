# FocusLock Web Prototype - Design Guidelines

## Design Approach
**Selected Approach:** Microsoft Fluent Design System - Aligns with the original Windows 11 native application design and provides consistent productivity-focused patterns.

**Key Design Principles:**
- Clean, task-oriented interface prioritizing functionality
- Subtle visual hierarchy supporting focused work sessions
- Consistent with Windows productivity applications
- Minimal distractions during active focus sessions

## Core Design Elements

### A. Color Palette
**Primary Colors:**
- Light mode: 210 15% 12% (deep blue-gray)
- Dark mode: 210 25% 88% (light blue-gray)
- Focus session active: 210 100% 50% (vibrant blue)

**Background Colors:**
- Light mode: 0 0% 98% (near white)
- Dark mode: 210 15% 8% (very dark blue-gray)

**Accent Colors:**
- Success (session complete): 120 45% 45% (muted green)
- Warning (time running low): 35 85% 55% (amber)

### B. Typography
**Font Family:** Segoe UI Variable (fallback: system-ui, sans-serif)

**Font Sizes & Weights:**
- Headers: 24px/32px, weight 600
- Body text: 14px/20px, weight 400
- Timer display: 48px/56px, weight 300
- Small labels: 12px/16px, weight 500

### C. Layout System
**Spacing Units:** Tailwind units 2, 4, 6, and 8 (8px, 16px, 24px, 32px)
- Component padding: p-4 or p-6
- Section margins: m-6 or m-8
- Element gaps: gap-2 or gap-4

### D. Component Library

**Core Components:**
- Large timer display with circular progress indicator
- Compact favorites bar with app icons (left sidebar)
- Instant search with filtered results
- Session control panel with preset duration buttons
- Blocklist management table with inline actions
- Settings panel with toggle switches and inputs

**Navigation:**
- Left sidebar for favorites (always visible)
- Top navigation bar with session status
- Modal overlays for settings and detailed views

**Forms:**
- Fluent-style input fields with subtle borders
- Toggle switches for binary settings
- Dropdown selectors for duration presets
- Search input with real-time filtering

**Data Displays:**
- Clean table layouts for rules management
- Card-based app selections in search results
- Status indicators for session state
- Progress bars for session countdown

**Overlays:**
- Modal dialogs for settings and confirmations
- Toast notifications for session milestones
- Dropdown menus for quick actions

### E. Animations
**Minimal Animation Strategy:**
- Smooth transitions (200ms) for state changes only
- Subtle fade-in for search results
- Timer countdown with gentle pulsing during final minutes
- No decorative animations that could distract during focus sessions

## Layout Structure
**Main Interface:**
- Left sidebar (200px): Favorites bar with pinned apps
- Center area: Large timer display and session controls
- Right panel: Search and rules management
- Top bar: Session status and quick settings access

**Focus on Functionality:**
- Immediate access to start/stop session controls
- Quick app search and blocking capabilities
- Clear visual feedback for active session state
- Minimal visual noise during active focus periods