import { invoke } from "@tauri-apps/api/core";
import { toast } from "svelte-sonner";
import { IsMobile } from "$lib/hooks/is-mobile.svelte";

type FileType = "json" | "lottie";

interface LottieFile {
  path: string;
  name: string;
  type: FileType;
  content: Uint8Array | null;
}

class AppState {
  // File State
  currentFile = $state<LottieFile | null>(null);
  isLoading = $state(false);
  isDragging = $state(false);
  error = $state<string | null>(null);

  // Playback State
  isPlaying = $state(true);
  loop = $state(true);
  speed = $state(1.0);
  currentFrame = $state(0);
  totalFrames = $state(0);
  duration = $state(0);

  // Visual State
  backgroundColor = $state("#0f1115");
  scaleMode = $state<"original" | "fit">("fit");
  isControlPanelOpen = $state(false);

  // Layout State
  #mobile = new IsMobile();
  get isMobile() {
    return this.#mobile.current;
  }

  // Metadata
  originalWidth = $state(0);
  originalHeight = $state(0);
  fps = $state(30); // Default to 30, will be updated on load

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
  }
}

export const appState = new AppState();
