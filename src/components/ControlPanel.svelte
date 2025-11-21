<script lang="ts">
  import { appState } from "$lib/state.svelte";

  function togglePlay() {
    appState.isPlaying = !appState.isPlaying;
  }
</script>

<div
  class="w-80 h-full bg-gray-50 border-l border-gray-200 flex flex-col shrink-0"
>
  <!-- Header -->
  <div class="p-4 border-b border-gray-200">
    <h2 class="font-semibold text-gray-800">Controls</h2>
  </div>

  <div class="flex-1 overflow-y-auto p-4 space-y-6">
    <!-- Playback -->
    <section class="space-y-3">
      <h3 class="text-xs font-bold text-gray-500 uppercase tracking-wider">
        Playback
      </h3>

      <div class="flex items-center justify-center gap-4">
        <button
          class="p-2 rounded-full hover:bg-gray-200 transition-colors"
          onclick={togglePlay}
          aria-label={appState.isPlaying ? "Pause" : "Play"}
        >
          {#if appState.isPlaying}
            <!-- Pause Icon -->
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              ><rect x="6" y="4" width="4" height="16"></rect><rect
                x="14"
                y="4"
                width="4"
                height="16"
              ></rect></svg
            >
          {:else}
            <!-- Play Icon -->
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              ><polygon points="5 3 19 12 5 21 5 3"></polygon></svg
            >
          {/if}
        </button>
      </div>

      <!-- Scrubber -->
      <div class="space-y-1">
        <input
          type="range"
          min="0"
          max="100"
          bind:value={appState.currentFrame}
          class="w-full accent-blue-600"
          disabled={!appState.currentFile}
        />
        <div class="flex justify-between text-xs text-gray-500">
          <span>0%</span>
          <span>100%</span>
        </div>
      </div>

      <div class="flex items-center justify-between text-sm">
        <span>Speed</span>
        <input
          type="number"
          bind:value={appState.speed}
          step="0.1"
          min="0.1"
          max="5"
          class="w-16 p-1 border border-gray-300 rounded text-right"
        />
      </div>

      <label class="flex items-center gap-2 text-sm cursor-pointer select-none">
        <input
          type="checkbox"
          bind:checked={appState.loop}
          class="rounded text-blue-600"
        />
        <span>Loop Animation</span>
      </label>
    </section>

    <hr class="border-gray-200" />

    <!-- Appearance -->
    <section class="space-y-3">
      <h3 class="text-xs font-bold text-gray-500 uppercase tracking-wider">
        Appearance
      </h3>

      <div class="space-y-2">
        <label class="block text-sm text-gray-700">Background</label>
        <div class="flex gap-2 items-center">
          <input
            type="color"
            bind:value={appState.backgroundColor}
            class="h-8 w-8 p-0 border-0 rounded cursor-pointer"
          />
          <input
            type="text"
            bind:value={appState.backgroundColor}
            class="flex-1 p-1 border border-gray-300 rounded text-sm uppercase"
          />
        </div>
      </div>

      <div class="space-y-2">
        <label class="block text-sm text-gray-700">Scale Mode</label>
        <div class="flex rounded-md shadow-sm isolate" role="group">
          <button
            class="flex-1 px-3 py-1.5 text-sm border border-gray-300 rounded-l-md hover:bg-gray-50 transition-colors {appState.scaleMode ===
            'fit'
              ? 'bg-blue-50 text-blue-700 border-blue-200 z-10'
              : 'bg-white text-gray-700'}"
            onclick={() => (appState.scaleMode = "fit")}
          >
            Fit
          </button>
          <button
            class="flex-1 px-3 py-1.5 text-sm border border-gray-300 rounded-r-md -ml-px hover:bg-gray-50 transition-colors {appState.scaleMode ===
            'original'
              ? 'bg-blue-50 text-blue-700 border-blue-200 z-10'
              : 'bg-white text-gray-700'}"
            onclick={() => (appState.scaleMode = "original")}
          >
            Original
          </button>
        </div>
      </div>
    </section>

    <hr class="border-gray-200" />

    <!-- Metadata -->
    <section class="space-y-3">
      <h3 class="text-xs font-bold text-gray-500 uppercase tracking-wider">
        Metadata
      </h3>

      {#if appState.currentFile}
        <div class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 text-sm">
          <span class="text-gray-500">Name</span>
          <span class="text-gray-900 truncate" title={appState.currentFile.name}
            >{appState.currentFile.name}</span
          >

          <span class="text-gray-500">Type</span>
          <span class="text-gray-900 uppercase"
            >{appState.currentFile.type}</span
          >

          <span class="text-gray-500">Size</span>
          <span class="text-gray-900"
            >{appState.currentFile.content?.length
              ? (appState.currentFile.content.length / 1024).toFixed(1)
              : 0} KB</span
          >
        </div>
      {:else}
        <p class="text-sm text-gray-400 italic">No file loaded</p>
      {/if}
    </section>
  </div>
</div>
