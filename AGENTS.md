# **AGENT CONTEXT: AnimaView \- Lottie Animation Viewer**

This document provides the foundational context for Large Language Models (LLMs) and development agents working on the 'AnimaView' project.

## **1\. Project Goal**

**AnimaView** is a cross-platform desktop application designed to provide a fast, reliable, and feature-rich previewer for Lottie animations. Its primary function is to accurately render Lottie files and offer a comprehensive set of inspection and control tools for designers and developers.

## **2\. Technology Stack**

|              Category |                      Technology |                                                                                         Key Constraint |
| --------------------: | ------------------------------: | -----------------------------------------------------------------------------------------------------: |
| **Application Shell** |                       **Tauri** |                                                Used for the native desktop integration (Rust backend). |
|       **Frontend UI** |          **Svelte / SvelteKit** |                          All components are highly reactive and follow Svelte 5's component lifecycle. |
|          **Language** |                  **TypeScript** |                                    Mandatory for both Rust command definitions and all frontend logic. |
|   **Lottie Playback** | `@lottiefiles/dotlottie-svelte` |                                               The core rendering engine for Lottie JSON/.lottie files. |
|           **Styling** |                **Tailwind CSS** |                                                         Used for all styling (utility-first approach). |
|      **Data Storage** |               None (Filesystem) | The app primarily reads local files. Persistence is limited to configuration settings if needed later. |

## **3\. Core Features & Functionality**

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

- **Background Color:** A color picker (or input field) to change the background color of the viewer area. This is a crucial feature for checking animation visibility on various backgrounds.
- **Scale/Fit Modes:** Buttons or selectors for displaying the animation at:
  - Original Size (100%).
  - Fit to Viewer Window (Aspect Ratio Maintained).
- **Metadata Display:** A read-only section showing key animation properties (e.g., Frame Rate (fps), Total Frames, Duration, Original Width/Height).

## **4\. Agent Instructions & Constraints**

1. **Tauri Command Priority (Rust-First):** All operations that touch the operating system or filesystem—specifically **reading file content**—must be encapsulated in asynchronous Tauri commands defined in the Rust (src-tauri) code. The frontend must only call these commands. **Do not use native browser file APIs (like FileReader) for files opened via the desktop UI.**
2. **Svelte Reactivity:** State management should leverage Svelte's reactivity ($state, $derived, etc..) to link UI controls (speed slider, color picker) directly to the Lottie viewer component's properties.
3. **Lottie Lifecycle:** The Svelte Viewer component must correctly manage the lottie-web instance, ensuring the animation is loaded and destroyed properly when the component is unmounted or a new file is loaded.
4. **Error Handling:** Implement graceful failure for file loading. If a .json file is malformed or a .lottie extraction fails, the UI must show a clear, user-friendly error message instead of crashing.

## **5\. MCP**

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
