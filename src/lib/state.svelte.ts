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

      const result = await invoke<[number[], string]>("export_animation", {
        filePath: this.currentFile.path,
        format,
        options: exportOptions,
      });

      const [data, extension] = result;
      const mimeType = format === "gif" ? "image/gif" : "video/mp4";
      const blob = new Blob([new Uint8Array(data)], { type: mimeType });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `animation.${extension}`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);

      toast.success("Export completed successfully!");
    } catch (e) {
      this.exportError = String(e);
      toast.error("Export failed: " + String(e));
    } finally {
      this.isExporting = false;
    }
  }
}

export const appState = new AppState();
