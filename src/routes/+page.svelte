<script lang="ts">
  import ControlPanel from "$lib/components/ControlPanel.svelte";
  import { ThemeSelector } from "$lib/components/ui-extra/theme-selector";
  import { Button } from "$lib/components/ui/button";
  import * as Drawer from "$lib/components/ui/drawer";
  import * as Sidebar from "$lib/components/ui/sidebar";
  import Viewer from "$lib/components/Viewer.svelte";
  import { appState } from "$lib/state.svelte";
  import TablerAdjustmentsHorizontal from "~icons/tabler/adjustments-horizontal";
  import TablerSettings from "~icons/tabler/settings";
  import TablerX from "~icons/tabler/x";

  // Auto-open sidebar on desktop when a file is added
  $effect(() => {
    if (appState.currentFile && !appState.isMobile) {
      appState.isControlPanelOpen = true;
    }
  });
</script>

<Sidebar.Provider
  bind:open={appState.isControlPanelOpen}
  class="h-screen w-screen overflow-hidden"
>
  <div class="flex h-full w-full bg-background">
    <!-- Main Content -->
    <div class="flex-1 relative flex flex-col items-center justify-center p-4">
      <!-- Viewer -->
      <div class="w-full h-full flex items-center justify-center relative">
        <Viewer />
      </div>

      <!-- Desktop Sidebar Trigger (if needed, but user wants sidebar to auto-appear) -->
      <!-- We can still add it if they want to manually hide it -->

      <div class="absolute top-5 left-5 flex gap-2">
        {#if appState.currentFile}
          <Button
            class="z-50 shadow-lg"
            variant="destructive"
            size="icon"
            onclick={() => appState.reset()}
            title="Close File"
          >
            <TablerX class="size-6" />
          </Button>

          {#if appState.isMobile}
            <Drawer.Root bind:open={appState.isControlPanelOpen}>
              <Drawer.Trigger asChild>
                <Button
                  variant="secondary"
                  size="icon"
                  class="z-50 shadow-lg"
                  title="Controls"
                >
                  <TablerAdjustmentsHorizontal class="size-6" />
                </Button>
              </Drawer.Trigger>
              <Drawer.Content class="h-[80vh]">
                <Drawer.Header class="border-b pb-4 px-6">
                  <div class="flex items-center gap-2">
                    <TablerSettings class="text-muted-foreground" />
                    <Drawer.Title>Controls</Drawer.Title>
                  </div>
                </Drawer.Header>
                <div class="overflow-y-auto flex-1">
                  <ControlPanel />
                </div>
              </Drawer.Content>
            </Drawer.Root>
          {:else}
            <Button
              variant="secondary"
              size="icon"
              class="z-50 shadow-lg"
              onclick={() =>
                (appState.isControlPanelOpen = !appState.isControlPanelOpen)}
              title="Toggle Controls"
            >
              <TablerAdjustmentsHorizontal class="size-6" />
            </Button>
          {/if}
        {/if}
      </div>

      <div class="absolute top-5 right-5 z-50">
        <ThemeSelector variant="ghost" />
      </div>
    </div>

    {#if !appState.isMobile}
      <Sidebar.Root side="right" variant="sidebar" collapsible="offcanvas">
        <Sidebar.Header
          class="p-5 border-b border-muted/20 flex items-center gap-3"
        >
          <TablerSettings class="text-muted-foreground" />
          <h2 class="font-semibold text-lg tracking-tight">Controls</h2>
        </Sidebar.Header>
        <Sidebar.Content>
          <ControlPanel />
        </Sidebar.Content>
      </Sidebar.Root>
    {/if}
  </div>
</Sidebar.Provider>
