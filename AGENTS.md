# **AGENT CONTEXT: AnimaView \- Lottie Animation Viewer**

This document provides the foundational context for Large Language Models (LLMs) and development agents working on the 'AnimaView' project.

## **1\. Project Goal**

**AnimaView** is a cross-platform desktop application designed to provide a fast, reliable, and feature-rich previewer for Lottie animations. Its primary function is to accurately render Lottie files and offer a comprehensive set of inspection and control tools for designers and developers.

## **2\. Technology Stack**

|               Category |                                                        Technology |                                                                                                            Key Constraint |
| ---------------------: | ----------------------------------------------------------------: | ------------------------------------------------------------------------------------------------------------------------: |
|  **Application Shell** |                                                         **Tauri** |                                                                   Used for the native desktop integration (Rust backend). |
|        **Frontend UI** |                                            **Svelte / SvelteKit** |                                             All components are highly reactive and follow Svelte 5's component lifecycle. |
|           **Language** |                                                    **TypeScript** |                                                       Mandatory for both Rust command definitions and all frontend logic. |
|    **Lottie Playback** |                                   `@lottiefiles/dotlottie-svelte` |                                                                  The core rendering engine for Lottie JSON/.lottie files. |
|            **Styling** |                                                  **Tailwind CSS** |                                                                            Used for all styling (utility-first approach). |
|              **Icons** |                                                  **Tabler Icons** | Used for all icons through uplugin-icons (e.g. `import TablerExclamationCircle from "~icons/tabler/exclamation-circle"`). |
|       **Data Storage** |                                                 None (Filesystem) |                    The app primarily reads local files. Persistence is limited to configuration settings if needed later. |
| **Package management** | **pnpm** for project root and **cargo** for `src-tauri` directory |                                                                      The app uses pnpm and cargo as its package managers. |
|      **UI Components** |                                                 **bits-ui / shadcn-svelte** |                                            Pre-built accessible UI components built on Radix UI primitives. |
|        **Theme System** |                                                   **mode-watcher** |                                                                 Handles system/light/dark theme switching with proper Svelte integration. |
|     **Notifications** |                                                  **svelte-sonner** |                                                                 Toast notifications for user feedback. |
|        **i18n System** |                                                     **Wuchale** |                                                               Localization system supporting English and Arabic (RTL support). |

## **2.5. Architecture**

The application follows a **multi-window architecture** with the following structure:

### **Window Types:**

| Window | Route | Purpose | Characteristics |
| ------ | ----- | ------- | --------------- |
| **Main** | `/` | Primary viewer area for animations | Resizable, drag-and-drop enabled, sidebar for controls |
| **Preferences** | `/preferences` | App settings (theme, language) | Fixed size (400x500), modal-like, non-resizable |
| **About** | `/about` | App information and credits | Fixed size (400x500), modal-like, no menu bar |

### **State Management:**

- **App State** (`src/lib/state.svelte.ts`): Manages file loading, playback state, visual state using Svelte 5 runes (`$state`, `$derived`)
- **Config Service** (`src/lib/config.svelte.ts`): Handles persistent configuration via Tauri commands (`get_config`, `set_config`)
- **Reactivity**: Changes propagate automatically through Svelte's reactivity system

### **Configuration Persistence:**

- Config is stored in OS-specific app config directory as `configurations.json`
- Supports: Theme (system/light/dark), Language, Canvas background colors (light/dark)
- Auto-saves with 300ms debounce
- Emits `config-updated` event to all windows for real-time sync

## **3. Core Features & Functionality**

The application is structured around a central **Viewer Pane** and a collapsible **Control Panel**.

### **A. Animation Loading & Handling**

1. **Input Methods:** Must support file selection (open dialog) and drag-and-drop onto the main window.
2. **File Formats:**
   - **Lottie JSON (.json):** Standard Lottie file.
   - **Lottie Binary (.lottie):** This is a ZIP container.

### **B. Playback Controls (Control Panel)**

The UI must provide the following interactivity:

- **Start/Stop/Pause:** Standard animation transport.
- **Loop Toggle:** A switch/checkbox to enable continuous playback.
- **Speed Control:** A numerical input or slider to adjust the playback rate (e.g., 0.1x to 5.0x).
- **Frame Scrubber:** A slider corresponding to the animation progress (frame-by-frame scrubbing).

### **C. Visual Customization & Inspection (Control Panel)**

- **Background Color:** A color picker (or input field) to change the background color of the viewer area. This is a crucial feature for checking animation visibility on various backgrounds. Colors are theme-aware (separate for light/dark themes).
- **Metadata Display:** A read-only section showing key animation properties (e.g., Frame Rate (fps), Total Frames, Duration, Original Width/Height, File size).

### **D. Window Management**

- **Multi-window Support:** Independent windows for main viewer, preferences, and about dialog
- **Window State:** Each window manages its own state, synchronized via Tauri events (`config-updated`, `menu-open`, `file-opened`)
- **Menu System:** Native application menu (File, Help) with keyboard shortcuts (Ctrl+O, Ctrl+P, Ctrl+Q)
- **File Associations:** OS-level file associations for .json and .lottie files

### **E. Keyboard Shortcuts**

- **Space:** Toggle play/pause (when animation loaded and not in input)
- **Escape:** Close current window or reset animation
- **Ctrl+W / Cmd+W:** Close window or reset animation (main window only)
- **F1:** Open About window
- **Ctrl+O / Cmd+O:** Open file dialog
- **Ctrl+P / Cmd+P:** Open Preferences window
- **Ctrl+Q / Cmd+Q:** Quit application

## **4. Internationalization**

- **Languages:** English (en), Arabic (ar) - RTL support for Arabic
- **Localization Files:** Located in `locales/` directory (`.po` format)
- **Auto-generation:** TypeScript types generated from `languages.json` via `scripts/sync-locales.ts`
- **Runtime Switching:** Language can be changed in Preferences, updates entire UI immediately
- **Wuchale Integration:** Load locales with `loadLocale()` and apply direction (`dir="rtl"` for Arabic)

## **5. Development Workflow**

### **Available Scripts:**

```bash
pnpm dev              # Start development server with locale sync
pnpm build            # Build for production with locale sync
pnpm tauri dev        # Start Tauri development
pnpm tauri build      # Build Tauri application
pnpm check            # Run svelte-check (type checking)
pnpm check:watch      # Watch mode for type checking
pnpm i18n:extract     # Extract translatable strings
pnpm i18n:clean       # Clean unused translation keys
```

### **Adding a New Language:**

1. Update `languages.json` with new language code
2. Run `pnpm dev` to auto-generate types in `src/lib/languages.ts`
3. A new `.po` file will be automatically added to `src/locales/` directory
4. Translate strings in that file

## **6. Agent Instructions & Constraints**

1. **Always use English in the code:** All variable names, function names, comments, and strings that are not part of the localization files must be in English.
2. **Tauri Command Priority (Rust-First):** All operations that touch the operating system or filesystem—specifically **reading file content**—must be encapsulated in asynchronous Tauri commands defined in the Rust (src-tauri) code. The frontend must only call these commands. **Do not use native browser file APIs (like FileReader) for files opened via the desktop UI.**
3. **Svelte Reactivity:** State management should leverage Svelte's reactivity ($state, $derived, etc..) to link UI controls (speed slider, color picker) directly to the Lottie viewer component's properties.
4. **Lottie Lifecycle:** The Svelte Viewer component must correctly manage the lottie-web instance, ensuring the animation is loaded and destroyed properly when the component is unmounted or a new file is loaded.
5. **Error Handling:** Implement graceful failure for file loading. If a .json file is malformed or a .lottie extraction fails, the UI must show a clear, user-friendly error message instead of crashing.
6. **Multi-Window Handling:** When working with multiple windows, always use window labels ("main", "preferences", "about") to identify and manage them. State synchronization between windows should use Tauri events (`emit` and `listen`).
7. **Theme-Aware UI:** When implementing UI components that display user-customizable colors (like the canvas background), ensure they respect the current theme setting (light/dark) and update appropriately when the theme changes.
8. **Configuration Persistence:** When adding new configuration options:
   - Define the type in both `AppConfig` (Rust: `src-tauri/src/lib.rs`) and `AppConfig` (TypeScript: `src/lib/config.svelte.ts`)
   - Provide sensible defaults in both locations
   - Use the auto-save mechanism (300ms debounce) in the ConfigService
   - Ensure all windows listen to the `config-updated` event for real-time updates

## **7. MCP**

You are able to use the Svelte MCP server, where you have access to comprehensive Svelte 5 and SvelteKit documentation. Here's how to use the available tools effectively:

### **Available MCP Tools:**

#### **1. list-sections**

Use this FIRST to discover all available documentation sections. Returns a structured list with titles, use_cases, and paths.
When asked about Svelte or SvelteKit topics, ALWAYS use this tool at the start of the chat to find relevant sections.

#### **2. get-documentation**

Retrieves full documentation content for specific sections. Accepts single or multiple sections.
After calling the list-sections tool, you MUST analyze the returned documentation sections (especially the use_cases field) and then use the get-documentation tool to fetch ALL documentation sections that are relevant for the user's task.

#### **3. svelte-autofixer**

Analyzes Svelte code and returns issues and suggestions.
You MUST use this tool whenever writing Svelte code before sending it to the user. Keep calling it until no issues or suggestions are returned.

#### **4. playground-link**

Generates a Svelte Playground link with the provided code.
After completing the code, ask the user if they want a playground link. Only call this tool after user confirmation and NEVER if code was written to files in their project.
