<script lang="ts">
  import ControlPanel from "$lib/components/ControlPanel.svelte";
  import { Button } from "$lib/components/ui/button";
  import Viewer from "$lib/components/Viewer.svelte";
  import { appState } from "$lib/state.svelte";
  import TablerX from "~icons/tabler/x";
</script>

<div class="flex h-screen w-screen">
  <!-- Main Content -->
  <div class="flex-1 relative flex flex-col">
    <!-- Viewer -->
    <div class="flex-1 relative">
      <Viewer />
    </div>

    {#if appState.currentFile}
      <Button
        class="z-50 absolute top-5 left-5"
        variant="destructive"
        size="icon"
        onclick={() => appState.reset()}
        ><TablerX class="size-6"></TablerX></Button
      >
    {/if}
  </div>
  <ControlPanel />
</div>
