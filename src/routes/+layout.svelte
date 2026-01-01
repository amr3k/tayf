<script lang="ts">
  import { Toaster } from "$lib/components/ui/sonner/index.js";
  import { appState } from "$lib/state.svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { fade } from "svelte/transition";
  import TablerUpload from "~icons/tabler/upload";
  import "@fontsource-variable/inter";
  import "@fontsource-variable/rubik";
  import "../app.css";

  let { children } = $props();

  $effect(() => {
    let unlisten: () => void;

    async function setupDragDrop() {
      const webview = getCurrentWebview();
      unlisten = await webview.onDragDropEvent((event) => {
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
    }

    setupDragDrop();

    return () => {
      if (unlisten) unlisten();
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
