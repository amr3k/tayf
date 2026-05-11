<script lang="ts">
  import { Toaster } from "$lib/components/ui/sonner/index.js";
  import { appConfig } from "$lib/config.svelte";
  import { appState } from "$lib/state.svelte";
  import "@fontsource-variable/inter/index.css";
  import "@fontsource-variable/rubik/index.css";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { open } from "@tauri-apps/plugin-dialog";
  import { ModeWatcher } from "mode-watcher";
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import { loadLocale } from "wuchale/load-utils";
  import TablerUpload from "~icons/tabler/upload";
  import "../app.css";
  import "../locales/main.loader.svelte.js";

  let { children } = $props();

  function syncLocale() {
    loadLocale(appConfig.lang);
    document.documentElement.lang = appConfig.lang;
    document.documentElement.dir = appConfig.lang === "ar" ? "rtl" : "ltr";
  }

  $effect(syncLocale);

  onMount(() => {
    let disposed = false;
    let unlistenDrag: (() => void) | undefined;
    let unlistenMenu: (() => void) | undefined;

    async function setupListeners() {
      const win = getCurrentWindow();
      if (win.label !== "main") return;

      const webview = getCurrentWebview();

      // Drag & Drop
      unlistenDrag = await webview.onDragDropEvent((event) => {
        if (event.payload.type === "enter" || event.payload.type === "over") {
          appState.isDragging = true;
        } else if (event.payload.type === "drop") {
          appState.isDragging = false;
          const paths = event.payload.paths;
          if (paths && paths.length > 0) {
            void appState.loadFile(paths[0]);
          }
        } else if (event.payload.type === "leave") {
          appState.isDragging = false;
        }
      });
      if (disposed) unlistenDrag();

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
            await appState.loadFile(selected);
          }
        } catch (e) {
          console.error("Failed to open file dialog:", e);
        }
      });
      if (disposed) unlistenMenu();
    }

    void setupListeners();

    return () => {
      disposed = true;
      unlistenDrag?.();
      unlistenMenu?.();
    };
  });
</script>

<ModeWatcher />
<Toaster position="bottom-left" />
{@render children()}

{#if appState.isDragging && getCurrentWindow().label === "main"}
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
