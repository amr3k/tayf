<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "$lib/components/ui/card";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Select from "$lib/components/ui/select";
  import { Separator } from "$lib/components/ui/separator";
  import { Switch } from "$lib/components/ui/switch";
  import { appState } from "$lib/state.svelte";
  import { toast } from "svelte-sonner";

  interface Props {
    show: boolean;
  }

  let { show = $bindable(false) }: Props = $props();

  let format = $state<"gif" | "mp4">("gif");
  let width = $state(800);
  let fps = $state(60);
  let loopGif = $state(true);
  let quality = $state("medium");
  let isExporting = $state(false);

  const qualityPresets = [
    { value: "low", label: "Low (smaller file)" },
    { value: "medium", label: "Medium (balanced)" },
    { value: "high", label: "High (better quality)" },
  ];

  const fpsOptions = [
    { value: 30, label: "30 FPS" },
    { value: 60, label: "60 FPS" },
  ];

  async function handleExport() {
    if (width < 100 || width > 4096) {
      toast.error("Width must be between 100 and 4096 pixels");
      return;
    }

    isExporting = true;

    const qualityValue =
      quality === "low" ? 60 : quality === "medium" ? 80 : 100;

    try {
      await appState.exportFile(format, {
        width,
        fps,
        loop_gif: loopGif,
        quality: qualityValue,
      });
      show = false;
    } finally {
      isExporting = false;
    }
  }

  function close() {
    if (!isExporting) {
      show = false;
    }
  }
</script>

{#if show}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
    onclick={(e: MouseEvent) => {
      if (e.target === e.currentTarget) close();
    }}
    role="presentation"
  >
    <Card class="w-112.5">
      <CardHeader>
        <CardTitle>Export Animation</CardTitle>
        <CardDescription
          >Configure export settings for your animation</CardDescription
        >
      </CardHeader>

      <CardContent class="space-y-6">
        <div class="space-y-3">
          <Label>Format</Label>
          <div class="flex gap-4">
            <label class="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                bind:group={format}
                value="gif"
                class="w-4 h-4"
              />
              <span class="text-sm">GIF</span>
            </label>
            <label class="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                bind:group={format}
                value="mp4"
                class="w-4 h-4"
              />
              <span class="text-sm">MP4</span>
            </label>
          </div>
        </div>

        <Separator />

        <div class="space-y-3">
          <Label for="width">Width (px)</Label>
          <Input
            type="number"
            id="width"
            bind:value={width}
            min="100"
            max="4096"
            step="10"
          />
          <p class="text-xs text-muted-foreground">
            Height will be auto-calculated to maintain aspect ratio
          </p>
        </div>

        <div class="space-y-3">
          <Label>Frame Rate</Label>
          <div class="flex gap-4">
            {#each fpsOptions as option}
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  type="radio"
                  bind:group={fps}
                  value={option.value}
                  class="w-4 h-4"
                />
                <span class="text-sm">{option.label}</span>
              </label>
            {/each}
          </div>
        </div>

        {#if format === "gif"}
          <div class="flex items-center justify-between">
            <Label for="loop">Loop Animation</Label>
            <Switch id="loop" bind:checked={loopGif} />
          </div>
        {/if}

        {#if format === "mp4"}
          <div class="space-y-3">
            <Label>Quality</Label>
            <Select.Root type="single" bind:value={quality}>
              <Select.Trigger class="w-full">
                {qualityPresets.find((q) => q.value === quality)?.label}
              </Select.Trigger>
              <Select.Content>
                {#each qualityPresets as preset}
                  <Select.Item value={preset.value} label={preset.label}>
                    {preset.label}
                  </Select.Item>
                {/each}
              </Select.Content>
            </Select.Root>
          </div>
        {/if}

        {#if isExporting}
          <div class="space-y-2">
            <Label>Exporting... {appState.exportProgress}%</Label>
            <div class="h-2 bg-muted rounded-full overflow-hidden">
              <div
                class="h-full bg-primary transition-all"
                style="width: {appState.exportProgress}%"
              ></div>
            </div>
          </div>
        {/if}
      </CardContent>

      <CardFooter class="justify-end gap-2">
        <Button variant="outline" onclick={close} disabled={isExporting}>
          Cancel
        </Button>
        <Button
          onclick={handleExport}
          disabled={isExporting || width < 100}
        >
          Export {format.toUpperCase()}
        </Button>
      </CardFooter>
    </Card>
  </div>
{/if}
