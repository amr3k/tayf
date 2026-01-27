import { appConfig } from "$lib/config.svelte";
import { IsMobile } from "$lib/hooks/is-mobile.svelte";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "svelte-sonner";

type FileType = "json" | "lottie";

interface LottieFile {
  path: string;
  name: string;
  type: FileType;
  content: Uint8Array | null;
}

interface ExportOptions {
  width: number;
  fps: number;
  loop_gif: boolean;
  quality: number;
}

class AppState {
  // File State
  currentFile = $state<LottieFile | null>(null);
  isLoading = $state(false);
  isDragging = $state(false);
  error = $state<string | null>(null);

  // Viewer reference for frame capture
  viewerRef = $state<{
    captureFrames: (
      totalFrames: number,
      fps: number,
      width: number,
      onProgress: (frame: number, dataUrl: string) => void
    ) => Promise<void>;
  } | null>(null);

  setViewerRef(ref: typeof this.viewerRef) {
    this.viewerRef = ref;
  }

  // Security: Max file size limit (100MB) (100 * 1024 * 1024)
  #MAX_FILE_SIZE = 104857600;

  // Metadata
  originalWidth = $state(0);
  originalHeight = $state(0);
  fps = $state(30); // Default to 30, will be updated on load

  // Playback State
  isPlaying = $state(true);
  loop = $state(true);
  speed = $state(1.0);
  currentFrame = $state(0);
  totalFrames = $state(0);
  duration = $state(0);

  // Layout State
  #mobile = new IsMobile();
  isControlPanelOpen = $state(false);

  // Export State
  isExporting = $state(false);
  exportProgress = $state(0);
  exportError = $state<string | null>(null);

  // Visual State
  get backgroundColor(): string {
    if (appConfig.theme === "light") {
      return appConfig.canvasBackgroundColor;
    }

    if (appConfig.theme === "dark") {
      return appConfig.canvasBackgroundColorDark;
    }

    // System theme - need to check actual system preference
    return this.#getSystemBackgroundColor();
  }

  set backgroundColor(value: string) {
    const theme = appConfig.theme;

    if (theme === "light") {
      appConfig.canvasBackgroundColor = value;
    } else if (theme === "dark") {
      appConfig.canvasBackgroundColorDark = value;
    } else {
      // System theme - check actual system preference
      if (this.#getSystemIsDark()) {
        appConfig.canvasBackgroundColorDark = value;
      } else {
        appConfig.canvasBackgroundColor = value;
      }
    }
  }

  #getSystemIsDark(): boolean {
    if (typeof window === "undefined") {
      return true;
    }

    return window.matchMedia("(prefers-color-scheme: dark)").matches;
  }

  #getSystemBackgroundColor(): string {
    if (typeof window === "undefined") {
      return "#0F1115";
    }

    const isDark = this.#getSystemIsDark();
    return isDark ? appConfig.canvasBackgroundColorDark : appConfig.canvasBackgroundColor;
  }

  get isMobile() {
    return this.#mobile.current;
  }

  async loadFile(path: string) {
    this.isLoading = true;
    this.error = null;
    this.totalFrames = 0;
    this.duration = 0;
    this.originalWidth = 0;
    this.originalHeight = 0;
    this.currentFrame = 0;
    this.isPlaying = true;

    try {
      const content: number[] = await invoke("read_file_content", {
        filePath: path,
      });

      if (content.length > this.#MAX_FILE_SIZE) {
        throw new Error(
          `File too large (${(content.length / 1024 / 1024).toFixed(1)}MB). Max size: 100MB`,
        );
      }

      const name = path.split(/[\\/]/).pop() || "animation";
      const extension = name.split(".").pop()?.toLowerCase();

      if (extension !== "json" && extension !== "lottie") {
        throw new Error("Unsupported file format");
      }

      this.currentFile = {
        path,
        name,
        type: extension as FileType,
        content: new Uint8Array(content),
      };

      if (!this.isMobile) {
        this.isControlPanelOpen = true;
      }
    } catch (e) {
      this.error = String(e);
      console.error("Failed to load file:", e);
    } finally {
      this.isLoading = false;
    }
  }

  async loadFromFile(file: File) {
    this.isLoading = true;
    this.error = null;
    this.totalFrames = 0;
    this.duration = 0;
    this.originalWidth = 0;
    this.originalHeight = 0;
    this.currentFrame = 0;
    this.isPlaying = true;
    try {
      const extension = file.name.split(".").pop()?.toLowerCase();

      if (extension !== "json" && extension !== "lottie") {
        throw new Error("Unsupported file format");
      }

      const arrayBuffer = await file.arrayBuffer();

      if (arrayBuffer.byteLength > this.#MAX_FILE_SIZE) {
        throw new Error(
          `File too large (${(arrayBuffer.byteLength / 1024 / 1024).toFixed(1)}MB). Max size: 100MB`,
        );
      }

      this.currentFile = {
        path: file.name, // We don't have the full path, use name
        name: file.name,
        type: extension as FileType,
        content: new Uint8Array(arrayBuffer),
      };

      if (!this.isMobile) {
        this.isControlPanelOpen = true;
      }
    } catch (e) {
      this.error = String(e);
      console.error("Failed to load file:", e);
      toast.error("Failed to load file: " + String(e));
    } finally {
      this.isLoading = false;
    }
  }

  reset() {
    this.currentFile = null;
    this.error = null;
    this.isPlaying = true;
    this.currentFrame = 0;
    this.isControlPanelOpen = false;
  }

  async exportFile(format: "gif" | "mp4", options?: ExportOptions) {
    if (!this.currentFile?.path) return;

    this.isExporting = true;
    this.exportProgress = 0;
    this.exportError = null;

    try {
      const exportOptions: ExportOptions = options || {
        width: 800,
        fps: 60,
        loop_gif: true,
        quality: 80,
      };

      let data: number[];
      let extension: string;

      // Determine which export method to use
      // If we have a viewer reference (regardless of how the file was loaded), use the capture method
      // The capture method uses the frontend's proper Lottie renderer
      if (this.viewerRef) {
        // Use the frontend's proper Lottie renderer to capture frames
        console.log("Using capture method for export (proper renderer)");
        const frames: string[] = [];
        const totalFramesToExport = Math.min(this.totalFrames || 100, 1024);

        await this.viewerRef.captureFrames(
          totalFramesToExport,
          exportOptions.fps,
          exportOptions.width,
          (frame, dataUrl) => {
            frames.push(dataUrl);
          },
        );

        const base64Frames = frames.map((url) => url.split(",")[1]);
        const result = await invoke<[number[], string]>("encode_frames", {
          format: format.toUpperCase(),
          frames: base64Frames,
          width: exportOptions.width,
          fps: exportOptions.fps,
          quality: exportOptions.quality,
        });
        const [data, extension] = result;

        // Check if we're in a Tauri environment
        const isTauri = typeof window.__TAURI_INTERNALS__ !== "undefined";

        if (isTauri) {
          // For Tauri, use the dialog plugin to get the file path and save the file using Tauri fs
          try {
            const { save } = await import("@tauri-apps/plugin-dialog");
            const { writeFile } = await import("@tauri-apps/plugin-fs");

            const filePath = await save({
              filters: [{
                name: format === "gif" ? "GIF Image" : "MP4 Video",
                extensions: [extension]
              }],
              defaultPath: `animation.${extension}`
            });

            if (filePath) {
              // Write the file using Tauri's file system plugin
              await writeFile(filePath, new Uint8Array(data));
              toast.success("Export completed successfully!");
            } else {
              // User cancelled the save dialog
              console.log("User cancelled the save dialog");
              return;
            }
          } catch (error) {
            console.error("Tauri export failed:", error);
            toast.error(`Export failed: ${String(error)}`);
            throw error;
          }
        } else if (window.showSaveFilePicker) {
          // Use modern File System Access API for web browsers
          const mimeType = format === "gif" ? "image/gif" : "video/mp4";
          const suggestedName = `animation.${extension}`;
          const fileHandle = await window.showSaveFilePicker({
            suggestedName,
            types: [
              {
                description: format === "gif" ? "GIF Image" : "MP4 Video",
                accept: { [mimeType]: [`.${extension}`] },
              },
            ],
          });

          const writable = await fileHandle.createWritable();
          await writable.write(new Uint8Array(data));
          await writable.close();
          toast.success("Export completed successfully!");
        } else {
          // Fallback to traditional download method for web browsers
          const mimeType = format === "gif" ? "image/gif" : "video/mp4";
          const blob = new Blob([data], { type: mimeType });
          const url = URL.createObjectURL(blob);
          const a = document.createElement("a");
          a.href = url;
          a.download = `animation.${extension}`;
          document.body.appendChild(a);
          a.click();
          document.body.removeChild(a);
          URL.revokeObjectURL(url);
          toast.success("Export completed successfully!");
        }
      } else {
        // We don't have a viewer reference, so we must use the Rust export method
        // Note: This may produce incomplete results due to the basic Rust renderer
        console.log("Using Rust export method (may have rendering issues)", this.currentFile?.path);
        const success: boolean = await invoke("export_animation", {
          filePath: this.currentFile.path,
          format,
          options: exportOptions,
        });

        if (success) {
          toast.success("Export completed successfully! (Note: May have rendering issues due to basic renderer)");
        } else {
          // User cancelled the save dialog
          console.log("User cancelled the save dialog");
        }
      }
    } catch (e) {
      if (typeof e === "object" && e && "name" in e && e.name === "AbortError") {
        return;
      }
      this.exportError = String(e);
      toast.error("Export failed: " + String(e));
    } finally {
      this.isExporting = false;
    }
  }
}

export const appState = new AppState();
