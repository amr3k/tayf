<script lang="ts">
  import { appState } from "$lib/state.svelte";
  import TablerFile from "~icons/tabler/file";
  import TablerPlayerPauseFilled from "~icons/tabler/player-pause-filled";
  import TablerPlayerPlayFilled from "~icons/tabler/player-play-filled";
  import TablerSettings from "~icons/tabler/settings";

  function togglePlay() {
    appState.isPlaying = !appState.isPlaying;
  }
</script>

<div
  class="w-80 h-full bg-slate-900/80 backdrop-blur-xl border-l border-slate-700/50 flex flex-col shrink-0 text-slate-200 shadow-2xl"
>
  <!-- Header -->
  <div class="p-5 border-b border-slate-700/50 flex items-center gap-3">
    <TablerSettings></TablerSettings>
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
            <TablerPlayerPauseFilled></TablerPlayerPauseFilled>
          {:else}
            <TablerPlayerPlayFilled></TablerPlayerPlayFilled>
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
          <label for="speed-input" class="text-xs text-slate-400 font-medium"
            >Speed</label
          >
          <div class="flex flex-col gap-2">
            <div
              class="flex items-center bg-slate-800/50 rounded-lg border border-slate-700/50 px-3 py-2"
            >
              <input
                id="speed-input"
                type="number"
                bind:value={appState.speed}
                step="0.1"
                min="0.1"
                max="5"
                class="w-full bg-transparent border-none text-sm text-white focus:ring-0 p-0"
                aria-label="Speed value"
              />
              <span class="text-xs text-slate-500">x</span>
            </div>
            <input
              type="range"
              bind:value={appState.speed}
              min="0.1"
              max="5"
              step="0.1"
              class="w-full h-1 bg-slate-700 rounded-lg appearance-none cursor-pointer"
              aria-label="Speed slider"
            />
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
          <TablerFile class="size-12"></TablerFile>
          <p class="text-sm italic">No file loaded</p>
        </div>
      {/if}
    </section>
  </div>
</div>
