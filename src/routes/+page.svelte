<script lang="ts">
  import Viewer from "../components/Viewer.svelte";
  import ControlPanel from "../components/ControlPanel.svelte";
  import { appState } from "$lib/state.svelte";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { fade, scale } from "svelte/transition";

  let isDragging = $state(false);

  onMount(() => {
    const unlisten = listen("tauri://drag-drop", (event: any) => {
      isDragging = false;
      if (event.payload.paths && event.payload.paths.length > 0) {
        appState.loadFile(event.payload.paths[0]);
      }
    });

    const unlistenEnter = listen("tauri://drag-enter", () => {
      isDragging = true;
    });

    const unlistenLeave = listen("tauri://drag-leave", () => {
      isDragging = false;
    });

    return () => {
      unlisten.then((f) => f());
      unlistenEnter.then((f) => f());
      unlistenLeave.then((f) => f());
    };
  });

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
      <Viewer />

      <!-- Drag Overlay -->
      {#if isDragging}
        <div
          class="absolute inset-0 bg-blue-500/20 border-4 border-blue-500/50 border-dashed z-50 flex items-center justify-center pointer-events-none backdrop-blur-sm transition-all"
          transition:fade={{ duration: 200 }}
        >
          <div
            class="bg-slate-900/90 p-8 rounded-2xl shadow-2xl text-blue-400 font-bold text-2xl flex flex-col items-center gap-4 backdrop-blur-xl border border-white/10"
            in:scale={{ start: 0.9, duration: 200 }}
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="48"
              height="48"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              ><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"
              ></path><polyline points="17 8 12 3 7 8"></polyline><line
                x1="12"
                y1="3"
                x2="12"
                y2="15"
              ></line></svg
            >
            Drop to Open
          </div>
        </div>
      {/if}
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

    <!-- Open Button (Floating) -->
    <button
      class="absolute top-4 left-4 bg-white/90 backdrop-blur shadow-sm border border-gray-200 px-4 py-2 rounded-lg text-sm font-medium text-gray-700 hover:bg-gray-50 transition-colors flex items-center gap-2 z-40"
      onclick={openFile}
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
        ><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"
        ></path><polyline points="14 2 14 8 20 8"></polyline><line
          x1="12"
          y1="18"
          x2="12"
          y2="12"
        ></line><line x1="9" y1="15" x2="15" y2="15"></line></svg
      >
      Open File
    </button>
  </div>

  <!-- Sidebar -->
  <ControlPanel />
</div>
