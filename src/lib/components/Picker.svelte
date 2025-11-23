<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { appState } from "$lib/state.svelte";
  import { cn } from "$lib/utils";
  import { open } from "@tauri-apps/plugin-dialog";
  import TablerUpload from "~icons/tabler/upload";

  let dragOver = $state(true);

  async function onFileSelect(file: string) {
    appState.loadFile(file);
  }

  const openFile = async () => {
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

      if (selected) {
        await onFileSelect(selected);
      }
    } catch (e) {
      console.error("Failed to open file dialog:", e);
    }
  };
</script>

<div class="size-full grid place-items-center">
  <label
    class="flex cursor-pointer border border-dashed p-8 rounded-xl border-muted-foreground flex-col items-center gap-4 {dragOver &&
      'drag-over'}"
  >
    <TablerUpload
      class={cn(
        "size-16 border border-dashed rounded-full p-3 border-muted-foreground",
        dragOver ? "" : "text-muted-foreground"
      )}
    />
    <h2 class="text-muted-foreground">
      Drag & drop lottie file here to preview
    </h2>
    <Button variant="default" onclick={openFile}>
      <TablerUpload class="size-4" />
      <span>Choose file</span>
    </Button>
  </label>
</div>

<style lang="postcss">
  @reference '../../app.css';
  .drag-over {
    @apply text-foreground border-foreground [&>svg]:border-foreground [&>h2]:text-foreground;
  }
</style>
