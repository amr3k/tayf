<script lang="ts">
  import Viewer from "../components/Viewer.svelte";
  import ControlPanel from "../components/ControlPanel.svelte";
  import { appState } from "$lib/state.svelte";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";

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

<div class="flex h-screen w-screen overflow-hidden bg-white">
  <!-- Main Content -->
  <div class="flex-1 relative flex flex-col">
    <!-- Viewer -->
    <div class="flex-1 relative">
      <Viewer />

      <!-- Drag Overlay -->
      {#if isDragging}
        <div
          class="absolute inset-0 bg-blue-500/20 border-4 border-blue-500 border-dashed z-50 flex items-center justify-center pointer-events-none"
        >
          <div
            class="bg-white p-6 rounded-xl shadow-xl text-blue-600 font-semibold text-xl"
          >
            Drop Lottie file here
          </div>
        </div>
      {/if}
    </div>

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
