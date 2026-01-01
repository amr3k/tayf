<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import TablerBrandGithub from "~icons/tabler/brand-github";
  import TablerWorld from "~icons/tabler/world";
  import { appInfo } from "$lib/app-info";

  const closeWindow = () => {
    getCurrentWindow().close();
  };
</script>

<div
  class="h-screen w-full bg-background text-foreground flex flex-col items-center justify-center p-8 text-center select-none"
  data-tauri-drag-region
>
  <div class="mb-6 relative group">
    <div
      class="absolute -inset-1 bg-linear-to-r from-primary to-purple-600 rounded-2xl blur opacity-25 group-hover:opacity-50 transition duration-1000 group-hover:duration-200"
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

  <div class="flex flex-col gap-3 w-full max-w-[200px]">
    <a
      href={appInfo.author.url}
      target="_blank"
      rel="noopener noreferrer"
      class="inline-flex items-center justify-center gap-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors h-9 px-4 py-2 rounded-md hover:bg-muted"
    >
      <TablerWorld class="size-4" />
      Visit Website
    </a>

    <Button variant="outline" class="w-full" onclick={closeWindow}>
      Close
    </Button>
  </div>

  <div class="absolute bottom-4 text-xs text-muted-foreground/50">
    Created by {appInfo.author.name}
  </div>
</div>
