<script lang="ts">
  import { Toaster } from "$lib/components/ui/sonner/index.js";
  import { appState } from "$lib/state.svelte";
  import "@fontsource-variable/inter";
  import "@fontsource-variable/rubik";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { open } from "@tauri-apps/plugin-dialog";
  import { fade } from "svelte/transition";
  import TablerUpload from "~icons/tabler/upload";
  import "../app.css";

  let { children } = $props();

  $effect(() => {
    let unlistenDrag: () => void;
    let unlistenMenu: () => void;

    async function setupListeners() {
      const webview = getCurrentWebview();

      // Drag & Drop
      unlistenDrag = await webview.onDragDropEvent((event) => {
        if (event.payload.type === "enter" || event.payload.type === "over") {
          appState.isDragging = true;
        } else if (event.payload.type === "drop") {
          appState.isDragging = false;
          const paths = event.payload.paths;
          if (paths && paths.length > 0) {
            appState.loadFile(paths[0]);
          }
        } else if (event.payload.type === "leave") {
          appState.isDragging = false;
        }
      });

      // Menu: Open File
      unlistenMenu = await listen("menu-open", async () => {
        try {
          const selected = await open({
            multiple: false,
            filters: [
              {
                name: "Lottie Animation",
                extensions: ["json", "lottie"],
              },
            ],
          });

          if (selected && typeof selected === "string") {
            appState.loadFile(selected);
          }
        } catch (e) {
          console.error("Failed to open file dialog:", e);
        }
      });
    }

    setupListeners();

    const handleKeydown = (e: KeyboardEvent) => {
      // Ctrl+W (or Cmd+W on Mac, handled by 'metaKey')
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "w") {
        e.preventDefault();
        if (appState.currentFile) {
          appState.reset();
        } else {
          getCurrentWindow().close();
        }
      }
    };

    window.addEventListener("keydown", handleKeydown);

    return () => {
      if (unlistenDrag) unlistenDrag();
      if (unlistenMenu) unlistenMenu();
      window.removeEventListener("keydown", handleKeydown);
    };
  });
</script>

<Toaster position="bottom-left" />
{@render children()}

{#if appState.isDragging}
  <div
    class="fixed inset-0 z-50 bg-background/80 backdrop-blur-sm flex flex-col items-center justify-center border-4 border-dashed border-primary m-4 rounded-3xl pointer-events-none"
    transition:fade={{ duration: 200 }}
  >
    <div
      class="bg-primary text-primary-foreground p-6 rounded-full shadow-2xl mb-4"
    >
      <TablerUpload class="size-12" />
    </div>
    <h2 class="text-2xl font-bold">Drop your animation here</h2>
    <p class="text-muted-foreground mt-2">Supports .json and .lottie files</p>
  </div>
{/if}
