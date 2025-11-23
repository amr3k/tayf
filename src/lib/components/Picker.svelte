<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { cn } from "$lib/utils";
  import { appState } from "$lib/state.svelte";

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
