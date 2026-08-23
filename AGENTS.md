# **AGENT CONTEXT: AnimaView \- Native Lottie Animation Viewer**

This document provides the foundational context for Large Language Models (LLMs) and development agents working on the 'AnimaView' project.

## **1\. Project Goal**

**AnimaView** is a 100% native Rust desktop application designed to provide a blazing-fast, lightweight, and feature-rich previewer for Lottie animations. Its primary function is to accurately render Lottie files using hardware/software vector rasterization (ThorVG) and offer a comprehensive set of inspection and control tools using GPUI.

## **2\. Technology Stack**

|               Category |                                                        Technology |                                                                                                            Key Constraint |
| ---------------------: | ----------------------------------------------------------------: | ------------------------------------------------------------------------------------------------------------------------: |
|   **Desktop GUI Engine** |                                                         **GPUI** |                                                                    High-performance GPU-accelerated 2D UI framework from Zed. |
|  **Lottie Vector Engine** |                                                       **ThorVG** |                                                C++ high-performance vector graphics engine with Lottie support via ThorVG FFI. |
|           **Language** |                                                         **Rust** |                                                      100% pure Rust desktop codebase (no Node.js / Webview / Web frontend). |
|       **Data Storage** |                                                 None (Filesystem) |                    The app primarily reads local files. Persistence is limited to configuration settings. |
| **Package Management** |                                                        **Cargo** |                                                                                 The app uses Cargo as its package manager. |
|        **Theme System** |                                                **Theme Palette** |                                                                 Handles system/light/dark theme switching natively in GPUI. |
|        **Export Engine** |                                                    **GIF / MP4** |                                                                 High-performance frame rasterizer to GIF and FFmpeg MP4 video. |
|        **i18n System** |                                                    **rust-i18n** |                                                               Localization system supporting English and Arabic (RTL support). |

## **2.5. Architecture**

The application is structured in clean modular Rust crates/modules:

### **Modules:**

| Module | Purpose | Characteristics |
| ------ | ------- | --------------- |
| `engine::lottie` | ThorVG Lottie rasterizer (`LoadedAnimation`) | C-FFI bindings to ThorVG, renders RGBA8888 frame buffers |
| `engine::dotlottie` | DotLottie archive extractor | Extracts ZIP container, parses manifest, retrieves JSON animation |
| `engine::metadata` | Metadata calculation and formatting | Dimensions, duration, FPS, total frames, file size |
| `export` | Animation export pipeline | GIF encoder (`gif` crate) and MP4 video encoder (`ffmpeg`) |
| `config` | App settings persistence | OS-specific `configurations.json` configuration manager |
| `i18n` | Internationalization | `rust-i18n` with runtime locale switching and RTL detection |
| `state` | AppState model | Playback state, transport controls, theme, active modal |
| `ui` | GPUI UI components | Canvas viewer, transport bar, sidebar, drop zone, modals |

### **State Management:**

- **App State** (`src/state.rs`): Manages loaded animation, frame scrubber, playback state (playing, loop, speed), theme, modals.
- **Config Service** (`src/config.rs`): Handles persistent configuration saved to OS config directory as `configurations.json`.

## **3. Core Features & Functionality**

The application is structured around a central **Viewer Canvas**, a **Transport Control Bar**, a collapsible **Sidebar**, and **Modal Dialogs**.

### **A. Animation Loading & Handling**

1. **Input Methods:** Supports file selection (native file dialog via `rfd`), CLI argument (`animaview <file>`), and drag-and-drop.
2. **File Formats:**
   - **Lottie JSON (.json):** Standard Lottie file format.
   - **Lottie Binary (.lottie):** ZIP package format with embedded manifest and assets.

### **B. Playback Controls**

The UI provides full transport controls:
- **Play/Pause:** Standard transport toggle with Space shortcut.
- **Frame Stepping:** Step backward and forward by 1 frame (or 10 frames with Shift/Ctrl).
- **Loop Toggle:** Switch for continuous playback looping.
- **Speed Control:** Speed selector chips (`0.5x`, `1.0x`, `1.5x`, `2.0x`).
- **Frame Scrubber:** Interactive progress track corresponding to animation frame and time.

### **C. Visual Customization & Inspection**

- **Canvas Background:** Theme-aware canvas background color customization with presets and custom hex colors.
- **Metadata Display:** Read-only inspection showing file name, format, file size, dimensions, FPS, duration, and frame count.
- **Exporting:** Export animations to animated GIF or MP4 video with customizable resolution, FPS, transparency, and looping.

### **D. Keyboard Shortcuts**

- **Space:** Toggle play/pause
- **Left / Right:** Step 1 frame backward / forward
- **Shift+Left / Shift+Right:** Step 10 frames backward / forward
- **Ctrl+W / Cmd+W (or Escape):** Close current file in preview or modal
- **Ctrl+Q / Cmd+Q (or Alt+F4):** Quit application
- **Ctrl+O / Cmd+O:** Open file dialog
- **Ctrl+P / Cmd+P:** Open Preferences modal
- **Ctrl+E / Cmd+E:** Open Export modal
- **Ctrl+B / Cmd+B:** Toggle Sidebar
- **F1:** Open About modal

## **4. Internationalization**

- **Languages:** English (`en`), Arabic (`ar`) - Full RTL support for Arabic
- **Localization Files:** Located in `locales/` directory (`en.json`, `ar.json`)
- **Runtime Switching:** Language can be changed in Preferences modal, updating the UI immediately.

## **5. Development Workflow**

### **Available Commands:**

```bash
cargo check           # Type checking
cargo test            # Run unit test suite
cargo run             # Run AnimaView in development
cargo run -- <file>   # Open animation directly
cargo build --release # Build optimized release binary
```

## **6. Agent Instructions & Constraints**

1. **Always use English in the code:** All variable names, function names, comments, and strings that are not part of localization files must be in English.
2. **Native Rust First:** Everything runs directly inside Rust and GPUI with native OS bindings. Do not introduce webview or node dependencies.
3. **ThorVG Safety:** When manipulating ThorVG FFI pointers (`tvg_...`), ensure memory safety, null checks, and cleanup in `Drop` implementations.
4. **Theme Awareness:** All UI components must use `ThemeColors` to render consistently in Dark, Light, and System modes.
