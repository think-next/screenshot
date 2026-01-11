# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a cross-platform desktop screenshot application built with **Tauri** (Rust backend + Vue 3 frontend). Users can capture screenshots, view them in fullscreen overlay, and select regions using drag-to-select functionality.

## Essential Commands

### Development
```bash
pnpm tauri dev          # Start development server (Vite on port 1420) and Tauri app
```

### Building
```bash
pnpm tauri build        # Build production executable
pnpm build              # Build Vue frontend only
```

### Testing
```bash
cd src-tauri && cargo test test_capture_screen    # Run screenshot capture unit test
```

### IDE Setup
Recommended VS Code extensions (configured in `.vscode/extensions.json`):
- Vue - Official
- Tauri
- rust-analyzer

## Architecture

### Frontend-Backend Communication
The app uses Tauri's IPC system:
- **Frontend** (Vue) invokes backend commands via `invoke()`
- **Backend** (Rust) registers commands in `lib.rs` using `#[tauri::command]`
- Events are emitted via Tauri events for overlay selection coordinates

### Key Components

**Frontend (`src/`):**
- `App.vue` - Main component managing screenshot display, window state, and fullscreen handling
- `public/overlay.html` - Standalone overlay window for drag-to-select region selection

**Backend (`src-tauri/src/`):**
- `lib.rs` - Tauri app initialization, logging setup, command registration
- `screenshot.rs` - Core screenshot capture using `xcap` crate with parallel RGBA→RGB conversion and JPEG encoding
- `main.rs` - Entry point that delegates to `screenshot_lib::run()`

### Screenshot Flow
1. User clicks "截屏" button → Main window hides (to avoid capturing itself)
2. Rust backend captures screen via `xcap`
3. Parallel RGBA to RGB conversion using Rayon
4. JPEG encoding (quality 100)
5. Base64 encoding for transport to frontend
6. Frontend displays as background image
7. Window shows and enters fullscreen
8. User can press Escape to exit fullscreen

## Important Implementation Details

### Performance Optimizations
- Parallel pixel conversion using Rayon (`screenshot.rs:48`)
- Pre-allocated buffers to avoid dynamic growth
- JPEG encoding for smaller payload size vs PNG
- Detailed timing logs for each operation step

### Window Management
- Escape key listener exits fullscreen (App.vue:65-72)
- Window hide/show cycle during screenshot to avoid capturing the app itself
- Fullscreen state tracking with `isFullscreen` ref

### Logging
- File-based logging at `~/Library/Application Support/screenshot-app/screenshot.log` (macOS)
- Format: `YYYY-MM-DD HH:MM:SS [LEVEL] - Message`
- Logs performance metrics for each screenshot operation step
- Falls back to stdout if file creation fails

### Capabilities and Permissions
Defined in `src-tauri/capabilities/default.json`:
- Window operations: `show`, `hide`, `set-focus`, `set-fullscreen`
- Image handling: `core:image:default`
- Basic Tauri APIs: `core:default`, `opener:default`

## Technology Stack

**Frontend:** Vue 3.5 (Composition API with `<script setup>`), TypeScript 5.6, Vite 6.0
**Backend:** Rust 2021, Tauri 2.0, xcap 0.4.1 (screen capture)
**Key Libraries:** rayon 1.8 (parallelism), jpeg-encoder 0.1, image 0.25.0

## Code Style Notes

- Chinese comments in Rust code explaining functionality
- Chinese comments in Vue code for UI elements
- TypeScript strict mode enabled
- Rust code includes comprehensive error handling and logging
