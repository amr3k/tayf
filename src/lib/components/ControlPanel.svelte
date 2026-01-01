<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Separator } from "$lib/components/ui/separator";
  import { Slider } from "$lib/components/ui/slider";
  import { Switch } from "$lib/components/ui/switch";
  import { Tabs, TabsList, TabsTrigger } from "$lib/components/ui/tabs";
  import { appState } from "$lib/state.svelte";
  import TablerFile from "~icons/tabler/file";
  import TablerPlayerPauseFilled from "~icons/tabler/player-pause-filled";
  import TablerPlayerPlayFilled from "~icons/tabler/player-play-filled";
  import TablerSettings from "~icons/tabler/settings";
</script>

<div
  class="w-80 h-full backdrop-blur-xl border-l border-muted/20 flex flex-col shrink-0 shadow-2xl bg-background/95"
>
  <!-- Header -->
  <div class="p-5 border-b border-muted/20 flex items-center gap-3">
    <TablerSettings class="text-muted-foreground" />
    <h2 class="font-semibold text-lg tracking-tight">Controls</h2>
  </div>

  <div class="flex-1 overflow-y-auto p-5 space-y-8 custom-scrollbar">
    <!-- Playback -->
    <section class="space-y-4">
      <div class="flex items-center justify-between">
        <h3
          class="text-xs font-bold uppercase tracking-wider text-muted-foreground"
        >
          Playback
        </h3>
        {#if appState.currentFile}
          <span
            class="text-xs font-mono px-2 py-0.5 rounded bg-muted text-muted-foreground"
          >
            {Math.round(appState.currentFrame)} / {Math.floor(
              appState.totalFrames
            )}
          </span>
        {/if}
      </div>

      <div class="flex items-center justify-center gap-6 py-2">
        <Button
          class="size-16 rounded-full shadow-lg hover:scale-105 active:scale-95 transition-transform"
          variant="default"
          onclick={() => (appState.isPlaying = !appState.isPlaying)}
          aria-label={appState.isPlaying ? "Pause" : "Play"}
        >
          {#if appState.isPlaying}
            <TablerPlayerPauseFilled class="size-8" />
          {:else}
            <TablerPlayerPlayFilled class="size-8" />
          {/if}
        </Button>
      </div>

      <!-- Scrubber -->
      <div class="space-y-3">
        <Slider
          type="multiple"
          value={[appState.currentFrame]}
          min={0}
          max={appState.totalFrames || 100}
          step={1}
          disabled={!appState.currentFile}
          onValueChange={(v: number[]) => (appState.currentFrame = v[0])}
          class="w-full"
        />
      </div>

      <div class="flex flex-col gap-6">
        <div class="space-y-3">
          <Label
            for="speed-input"
            class="text-xs text-muted-foreground font-medium">Speed</Label
          >
          <div class="flex items-center gap-4">
            <div class="relative w-32 shrink-0">
              <Input
                id="speed-input"
                type="number"
                value={appState.speed}
                step="0.1"
                min="0.1"
                max="5"
                class="pr-8 font-mono text-sm"
                oninput={(e: Event & { currentTarget: HTMLInputElement }) =>
                  (appState.speed = Number(e.currentTarget.value))}
                aria-label="Speed value"
              />
              <span
                class="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-muted-foreground"
                >x</span
              >
            </div>
            <Slider
              type="multiple"
              value={[appState.speed]}
              min={0.1}
              max={5}
              step={0.1}
              onValueChange={(v: number[]) => (appState.speed = v[0])}
              class="flex-1"
            />
          </div>
        </div>
        <Label
          for="loop-mode"
          class="text-sm cursor-pointer font-medium flex items-center justify-between w-full border border-muted-foreground/20 rounded-lg p-3 bg-muted/30"
        >
          <span>Loop Playback</span>
          <Switch
            id="loop-mode"
            checked={appState.loop}
            onCheckedChange={(v: boolean) => (appState.loop = v)}
          />
        </Label>
      </div>
    </section>

    <Separator />

    <!-- Appearance -->
    <section class="space-y-4">
      <h3
        class="text-xs font-bold text-muted-foreground uppercase tracking-wider"
      >
        Appearance
      </h3>

      <div class="space-y-3">
        <Label class="text-xs text-muted-foreground font-medium"
          >Background</Label
        >
        <div class="flex gap-3 items-center">
          <div
            class="relative w-10 h-10 rounded-md overflow-hidden ring-1 ring-border shadow-sm"
          >
            <input
              type="color"
              bind:value={appState.backgroundColor}
              class="absolute inset-0 w-[150%] h-[150%] -top-1/4 -left-1/4 p-0 border-0 cursor-pointer"
            />
          </div>
          <Input
            type="text"
            bind:value={appState.backgroundColor}
            class="flex-1 font-mono uppercase"
          />
        </div>
      </div>

      <div class="space-y-3">
        <Label class="text-xs text-muted-foreground font-medium"
          >Scale Mode</Label
        >
        <Tabs
          value={appState.scaleMode}
          onValueChange={(v: string) =>
            (appState.scaleMode = v as "fit" | "original")}
          class="w-full"
        >
          <TabsList class="w-full grid grid-cols-2">
            <TabsTrigger value="fit">Fit</TabsTrigger>
            <TabsTrigger value="original">Fill</TabsTrigger>
          </TabsList>
        </Tabs>
      </div>
    </section>

    <Separator />

    <!-- Metadata -->
    <section class="space-y-4">
      <h3
        class="text-xs font-bold text-muted-foreground uppercase tracking-wider"
      >
        Metadata
      </h3>

      {#if appState.currentFile}
        <div class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-3 text-sm">
          <span class="text-muted-foreground">Name</span>
          <span class="truncate font-medium" title={appState.currentFile.name}>
            {appState.currentFile.name}
          </span>

          <span class="text-muted-foreground">Type</span>
          <span
            class="font-mono text-xs bg-muted px-1.5 py-0.5 rounded w-fit uppercase"
          >
            {appState.currentFile.type}
          </span>

          <span class="text-muted-foreground">Size</span>
          <span>
            {appState.currentFile.content?.length
              ? (appState.currentFile.content.length / 1024).toFixed(1)
              : 0} KB
          </span>

          <span class="text-muted-foreground">Dimensions</span>
          <span class="font-mono">
            {appState.originalWidth} x {appState.originalHeight}
          </span>

          <span class="text-muted-foreground">FPS</span>
          <span class="font-mono">{appState.fps.toFixed(2)}</span>

          <span class="text-muted-foreground">Duration</span>
          <span class="font-mono">{appState.duration.toFixed(2)}s</span>

          <span class="text-muted-foreground">Frames</span>
          <span class="font-mono">{Math.round(appState.totalFrames)}</span>
        </div>
      {:else}
        <div
          class="flex flex-col items-center justify-center py-8 text-muted-foreground gap-2 border-2 border-dashed border-muted rounded-lg"
        >
          <TablerFile class="size-10 opacity-50" />
          <p class="text-sm italic">No file loaded</p>
        </div>
      {/if}
    </section>
  </div>
</div>
