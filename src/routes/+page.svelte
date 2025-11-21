<script lang="ts">
  import { appState } from "$lib/state.svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import TablerExclamationCircle from "~icons/tabler/exclamation-circle";
  import TablerX from "~icons/tabler/x";
  import ControlPanel from "../components/ControlPanel.svelte";
  import Viewer from "../components/Viewer.svelte";

  async function openFile() {
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
  }
</script>

<div
  class="flex h-screen w-screen overflow-hidden bg-slate-950 text-slate-200 selection:bg-blue-500/30"
>
  <!-- Main Content -->
  <div class="flex-1 relative flex flex-col">
    <!-- Viewer -->
    <div
      class="flex-1 relative bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjAiIGhlaWdodD0iMjAiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI+PGNpcmNsZSBjeD0iMSIgY3k9IjEiIHI9IjEiIGZpbGw9InJnYmEoMjU1LDI1NSwyNTUsMC4wNSkiLz48L3N2Zz4=')]"
    >
      <Viewer {openFile} />
    </div>

    {#if appState.currentFile}
      <button
        class="absolute top-5 left-5 bg-white/90 shadow-sm border border-gray-200 p-1 hover:scale-125 rounded-full text-sm font-medium text-gray-700 hover:bg-red-50 hover:text-red-500 cursor-pointer transition-all z-50"
        onclick={() => appState.reset()}
      >
        <TablerX class="size-6"></TablerX>
      </button>
    {/if}

    {#if appState.error}
      <div
        class="absolute bottom-4 left-4 right-4 z-50 flex justify-center pointer-events-none"
      >
        <div
          class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg shadow-lg flex items-center gap-3 pointer-events-auto max-w-xl"
        >
          <TablerExclamationCircle class="size-5"></TablerExclamationCircle>
          <span class="flex-1 text-sm">{appState.error}</span>
          <button
            class="p-1 hover:bg-red-100 rounded"
            onclick={() => (appState.error = null)}
            aria-label="Dismiss error"
          >
            <TablerX class="size-4"></TablerX>
          </button>
        </div>
      </div>
    {/if}
  </div>

  <!-- Sidebar -->
  <ControlPanel />
</div>
