<script lang="ts">
  import { appInfo } from "$lib/app-info";
  import { Button } from "$lib/components/ui/button";
  import TablerBrandGithub from "~icons/tabler/brand-github";
  import TablerWorld from "~icons/tabler/world";
</script>

<div
  class="h-screen w-full bg-background text-foreground flex flex-col items-center justify-center p-8 text-center select-none"
  data-tauri-drag-region
>
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
    <Button variant="ghost" href={appInfo.author.url}>
      <TablerBrandGithub class="size-4" />
      <span>Github</span>
    </Button>
    <Button variant="ghost" href={appInfo.author.url}>
      <TablerWorld class="size-4" />
      <span>Author Website</span>
    </Button>
  </div>

  <div class="absolute bottom-4 text-xs text-muted-foreground/50">
    Created by {appInfo.author.name}
  </div>
</div>
