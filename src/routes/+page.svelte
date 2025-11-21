<script lang="ts">
  import Viewer from "../components/Viewer.svelte";
  import ControlPanel from "../components/ControlPanel.svelte";
  import { appState } from "$lib/state.svelte";
  import { open } from "@tauri-apps/plugin-dialog";

  let isDragging = $state(false);

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

    {#if appState.error}
      <div
        class="absolute bottom-4 left-4 right-4 z-50 flex justify-center pointer-events-none"
      >
        <div
          class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg shadow-lg flex items-center gap-3 pointer-events-auto max-w-xl"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            ><circle cx="12" cy="12" r="10"></circle><line
              x1="12"
              y1="8"
              x2="12"
              y2="12"
            ></line><line x1="12" y1="16" x2="12.01" y2="16"></line></svg
          >
          <span class="flex-1 text-sm">{appState.error}</span>
          <button
            class="p-1 hover:bg-red-100 rounded"
            onclick={() => (appState.error = null)}
            aria-label="Dismiss error"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              ><line x1="18" y1="6" x2="6" y2="18"></line><line
                x1="6"
                y1="6"
                x2="18"
                y2="18"
              ></line></svg
            >
          </button>
        </div>
      </div>
    {/if}
  </div>

  <!-- Sidebar -->
  <ControlPanel />
</div>
