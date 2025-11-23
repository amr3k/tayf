<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { appState } from "$lib/state.svelte";
  import { cn } from "$lib/utils";
  import { open } from "@tauri-apps/plugin-dialog";
  import { toast } from "svelte-sonner";
  import TablerUpload from "~icons/tabler/upload";

  let isDragging = $state(false);

  async function onFileSelect(file: string | File) {
    if (typeof file === "string") {
      appState.loadFile(file);
    } else if (file instanceof File) {
      appState.loadFromFile(file);
    }
  }

  function onDragOver(e: DragEvent) {
    e.preventDefault();
    isDragging = true;
  }

  function onDragLeave() {
    isDragging = false;
  }

  async function onDrop(e: DragEvent) {
    e.preventDefault();
    isDragging = false;

    const droppedFile = e.dataTransfer?.files[0];
    if (droppedFile) {
      appState.loadFromFile(droppedFile);
    } else {
      toast.error("No file dropped");
    }
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

<div
  class="size-full grid place-items-center"
  role="region"
  aria-label="File Drop Zone"
  ondragover={onDragOver}
  ondragleave={onDragLeave}
  ondrop={onDrop}
>
  <label
    class="flex cursor-pointer text-muted-foreground border border-dashed p-8 rounded-xl border-muted-foreground flex-col items-center gap-4 {isDragging &&
      'drag-over'}"
  >
    <TablerUpload
      class={cn(
        "size-16 border border-dashed rounded-full p-3 border-muted-foreground"
      )}
    />
    <h2>Drag & drop lottie file here to preview</h2>
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
