<!--
	Installed from @ieedan/shadcn-svelte-extras
-->

<script lang="ts">
  import { appState } from "$lib/state.svelte";
  import { cn } from "$lib/utils/utils";
  import UploadIcon from "@lucide/svelte/icons/upload";
  import { open } from "@tauri-apps/plugin-dialog";
  import { useId } from "bits-ui";
  import type { FileDropZoneProps } from "./types";

  let {
    id = useId(),
    children,
    maxFileSize,
    fileCount,
    disabled = false,
    onFileRejected,
    accept,
    class: className,
    ...rest
  }: FileDropZoneProps = $props();

  const drop = async (
    e: DragEvent & {
      currentTarget: EventTarget & HTMLLabelElement;
    }
  ) => {
    if (disabled) return;

    e.preventDefault();
    const droppedFile = e.dataTransfer?.files?.[0];
    if (droppedFile) {
      appState.loadFromFile(droppedFile);
    }
  };

  const openFile = async () => {
    if (disabled) return;

    try {
      const selected = await open({
        multiple: Number(fileCount) > 1,
        filters: [
          {
            name: "Lottie Animation",
            extensions: ["json", "lottie"],
          },
        ],
      });

      if (selected) {
        if (selected && typeof selected === "string") {
          appState.loadFile(selected);
        }
      }
    } catch (e) {
      console.error("Failed to open file dialog:", e);
    }
  };
</script>

<label
  ondragover={(e) => e.preventDefault()}
  ondrop={drop}
  for={id}
  class={cn(
    "border-border hover:bg-accent/25 flex h-48 w-full place-items-center justify-center rounded-lg border-2 border-dashed p-6 transition-all hover:cursor-pointer aria-disabled:opacity-50 aria-disabled:hover:cursor-not-allowed",
    className
  )}
>
  {#if children}
    {@render children()}
  {:else}
    <div class="flex flex-col place-items-center justify-center gap-2">
      <div
        class="border-border text-muted-foreground flex size-14 place-items-center justify-center rounded-full border border-dashed"
      >
        <UploadIcon class="size-7" />
      </div>
      <div class="flex flex-col gap-0.5 text-center">
        <p class="text-muted-foreground font-medium">
          Drag & drop files here, or click to select files
        </p>
        <p class="text-muted-foreground/75 text-sm">You can select 1 file</p>
      </div>
    </div>
  {/if}
  <button {...rest} {id} type="button" onclick={openFile} class="hidden"
  ></button>
</label>
