<script lang="ts">
  import { appState } from "$lib/state.svelte";

  function togglePlay() {
    appState.isPlaying = !appState.isPlaying;
  }
</script>

<div
  class="w-80 h-full bg-slate-900/80 backdrop-blur-xl border-l border-slate-700/50 flex flex-col shrink-0 text-slate-200 shadow-2xl"
>
  <!-- Header -->
  <div class="p-5 border-b border-slate-700/50 flex items-center gap-3">
    <div
      class="w-8 h-8 rounded-lg bg-blue-500/20 flex items-center justify-center text-blue-400"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="18"
        height="18"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        ><circle cx="12" cy="12" r="10"></circle><line
          x1="12"
          y1="16"
          x2="12"
          y2="12"
        ></line><line x1="12" y1="8" x2="12.01" y2="8"></line></svg
      >
    </div>
    <h2 class="font-semibold text-lg tracking-tight text-white">Controls</h2>
  </div>

  <div class="flex-1 overflow-y-auto p-5 space-y-8 custom-scrollbar">
    <!-- Playback -->
    <section class="space-y-4">
      <div class="flex items-center justify-between">
        <h3 class="text-xs font-bold text-slate-500 uppercase tracking-wider">
          Playback
        </h3>
        {#if appState.currentFile}
          <span
            class="text-xs font-mono text-blue-400 bg-blue-500/10 px-2 py-0.5 rounded"
          >
            {Math.round(appState.currentFrame)} / {appState.totalFrames}
          </span>
        {/if}
      </div>

      <div class="flex items-center justify-center gap-6 py-2">
        <button
          class="w-14 h-14 rounded-full bg-blue-600 hover:bg-blue-500 text-white shadow-lg shadow-blue-900/20 flex items-center justify-center transition-all hover:scale-105 active:scale-95"
          onclick={togglePlay}
          aria-label={appState.isPlaying ? "Pause" : "Play"}
        >
          {#if appState.isPlaying}
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="currentColor"
              stroke="none"
              ><rect x="6" y="4" width="4" height="16" rx="1"></rect><rect
                x="14"
                y="4"
                width="4"
                height="16"
                rx="1"
              ></rect></svg
            >
          {:else}
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="currentColor"
              stroke="none"><path d="M5 3l14 9-14 9V3z"></path></svg
            >
          {/if}
        </button>
      </div>

      <!-- Scrubber -->
      <div class="space-y-2">
        <input
          type="range"
          min="0"
          max={appState.totalFrames || 100}
          bind:value={appState.currentFrame}
          class="w-full"
          disabled={!appState.currentFile}
        />
      </div>

      <div class="grid grid-cols-2 gap-4">
        <div class="space-y-1.5">
          <label class="text-xs text-slate-400 font-medium">Speed</label>
          <div
            class="flex items-center bg-slate-800/50 rounded-lg border border-slate-700/50 px-3 py-2"
          >
            <input
              type="number"
              bind:value={appState.speed}
              step="0.1"
              min="0.1"
              max="5"
              class="w-full bg-transparent border-none text-sm text-white focus:ring-0 p-0"
            />
            <span class="text-xs text-slate-500">x</span>
          </div>
        </div>

        <div class="flex items-end pb-2">
          <label
            class="flex items-center gap-3 cursor-pointer group select-none"
          >
            <div class="relative">
              <input
                type="checkbox"
                bind:checked={appState.loop}
                class="sr-only peer"
              />
              <div
                class="w-10 h-6 bg-slate-700 rounded-full peer-checked:bg-blue-600 transition-colors"
              ></div>
              <div
                class="absolute left-1 top-1 w-4 h-4 bg-white rounded-full transition-transform peer-checked:translate-x-4"
              ></div>
            </div>
            <span
              class="text-sm text-slate-300 group-hover:text-white transition-colors"
              >Loop</span
            >
          </label>
        </div>
      </div>
    </section>

    <div class="h-px bg-slate-700/50"></div>

    <!-- Appearance -->
    <section class="space-y-4">
      <h3 class="text-xs font-bold text-slate-500 uppercase tracking-wider">
        Appearance
      </h3>

      <div class="space-y-2">
        <label class="text-xs text-slate-400 font-medium">Background</label>
        <div
          class="flex gap-3 items-center bg-slate-800/50 p-2 rounded-lg border border-slate-700/50"
        >
          <div
            class="relative w-8 h-8 rounded-md overflow-hidden ring-1 ring-slate-600/50"
          >
            <input
              type="color"
              bind:value={appState.backgroundColor}
              class="absolute inset-0 w-[150%] h-[150%] -top-1/4 -left-1/4 p-0 border-0 cursor-pointer"
            />
          </div>
          <input
            type="text"
            bind:value={appState.backgroundColor}
            class="flex-1 bg-transparent border-none text-sm font-mono text-slate-300 focus:ring-0 uppercase p-0"
          />
        </div>
      </div>

      <div class="space-y-2">
        <label class="text-xs text-slate-400 font-medium">Scale Mode</label>
        <div
          class="flex bg-slate-800/50 p-1 rounded-lg border border-slate-700/50"
        >
          <button
            class="flex-1 px-3 py-1.5 text-xs font-medium rounded-md transition-all {appState.scaleMode ===
            'fit'
              ? 'bg-slate-600 text-white shadow-sm'
              : 'text-slate-400 hover:text-slate-200'}"
            onclick={() => (appState.scaleMode = "fit")}
          >
            Fit
          </button>
          <button
            class="flex-1 px-3 py-1.5 text-xs font-medium rounded-md transition-all {appState.scaleMode ===
            'original'
              ? 'bg-slate-600 text-white shadow-sm'
              : 'text-slate-400 hover:text-slate-200'}"
            onclick={() => (appState.scaleMode = "original")}
          >
            Original
          </button>
        </div>
      </div>
    </section>

    <div class="h-px bg-slate-700/50"></div>

    <!-- Metadata -->
    <section class="space-y-4">
      <h3 class="text-xs font-bold text-slate-500 uppercase tracking-wider">
        Metadata
      </h3>

      {#if appState.currentFile}
        <div class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-3 text-sm">
          <span class="text-slate-500">Name</span>
          <span
            class="text-slate-200 truncate font-medium"
            title={appState.currentFile.name}>{appState.currentFile.name}</span
          >

          <span class="text-slate-500">Type</span>
          <span
            class="text-slate-200 uppercase font-mono text-xs bg-slate-800 px-1.5 py-0.5 rounded w-fit"
            >{appState.currentFile.type}</span
          >

          <span class="text-slate-500">Size</span>
          <span class="text-slate-200"
            >{appState.currentFile.content?.length
              ? (appState.currentFile.content.length / 1024).toFixed(1)
              : 0} KB</span
          >

          <span class="text-slate-500">Dimensions</span>
          <span class="text-slate-200 font-mono"
            >{appState.originalWidth} x {appState.originalHeight}</span
          >

          <span class="text-slate-500">FPS</span>
          <span class="text-slate-200 font-mono">{appState.fps.toFixed(2)}</span
          >

          <span class="text-slate-500">Duration</span>
          <span class="text-slate-200 font-mono"
            >{appState.duration.toFixed(2)}s</span
          >

          <span class="text-slate-500">Frames</span>
          <span class="text-slate-200 font-mono">{appState.totalFrames}</span>
        </div>
      {:else}
        <div
          class="flex flex-col items-center justify-center py-8 text-slate-600 gap-2"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="32"
            height="32"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
            ><path
              d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"
            ></path><polyline points="14 2 14 8 20 8"></polyline></svg
          >
          <p class="text-sm italic">No file loaded</p>
        </div>
      {/if}
    </section>
  </div>
</div>
