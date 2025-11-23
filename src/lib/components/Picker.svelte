<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { appState } from "$lib/state.svelte";
  import { cn } from "$lib/utils";
  import { open } from "@tauri-apps/plugin-dialog";
  import TablerUpload from "~icons/tabler/upload";

  type Props = {
    class?: string;
  };

  let { class: classess = "" }: Props = $props();

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

<div class={cn("w-full", classess)}>
  <Button variant="default" onclick={openFile}>
    <TablerUpload class="size-4" />
    <span>Choose file</span>
  </Button>
</div>
