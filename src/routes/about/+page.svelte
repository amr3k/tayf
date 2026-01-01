<script lang="ts">
  import { appInfo } from "$lib/app-info";
  import { Button } from "$lib/components/ui/button";
  import TablerBrandGithub from "~icons/tabler/brand-github";
  import TablerWorld from "~icons/tabler/world";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";

  function handleKeydown(e: KeyboardEvent) {
    const isW = (e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "w";
    const isEsc = e.key === "Escape" || e.key === "Esc" || e.keyCode === 27;

    if (isW || isEsc) {
      e.preventDefault();
      e.stopPropagation();
      getCurrentWindow().close();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="h-screen w-full bg-background text-foreground flex flex-col items-center justify-center p-8 text-center select-none outline-hidden relative"
>
  <!-- Drag Region -->
  <div class="absolute top-0 left-0 w-full h-8" data-tauri-drag-region></div>

  <div class="mb-6 relative group rounded-2xl">
    <div
      class="absolute -inset-1 bg-linear-to-r from-primary to-secondar rounded-2xl blur opacity-25 group-hover:opacity-50 transition duration-1000 group-hover:duration-200"
    ></div>
    <img
      src="/favicon.png"
      alt="AnimaView Logo"
      class="relative size-24 rounded-xl shadow-xl border border-border/50 bg-background/50 backdrop-blur-sm p-4"
    />
  </div>

  <h1 class="text-2xl font-bold tracking-tight mb-2 flex items-center gap-2">
    {appInfo.name}
    <span
      class="text-xs font-normal px-2 py-0.5 rounded-full bg-muted text-muted-foreground border border-border"
      >v{appInfo.version}</span
    >
  </h1>

  <p class="text-muted-foreground text-sm max-w-xs mb-8 leading-relaxed">
    {appInfo.description}
  </p>

  <div class="flex gap-3">
    <Button variant="ghost" href={appInfo.author.url} target="_blank">
      <TablerBrandGithub class="size-4" />
      <span>Github</span>
    </Button>
    <Button variant="ghost" href={appInfo.author.url} target="_blank">
      <TablerWorld class="size-4" />
      <span>Author Website</span>
    </Button>
  </div>

  <div class="absolute bottom-4 text-xs text-muted-foreground/50">
    Created by {appInfo.author.name}
  </div>
</div>
