<script lang="ts">
  import {
    FileDropZone,
    type FileDropZoneProps,
  } from "$lib/components/ui-extra/file-drop-zone";
  import { appState } from "$lib/state.svelte";
  import { DotLottieSvelte } from "@lottiefiles/dotlottie-svelte";
  import { toast } from "svelte-sonner";
  import { fade } from "svelte/transition";

  let dotLottie: any = $state(null);

  $effect(() => {
    if (dotLottie) {
      if (appState.isPlaying) {
        dotLottie.play();
      } else {
        dotLottie.pause();
      }
      dotLottie.setSpeed(appState.speed);

      // Only seek if the frame difference is significant to avoid fighting with playback
      // or if we are paused (scrubbing)
      const current = dotLottie.currentFrame;
      if (
        !appState.isPlaying ||
        Math.abs(current - appState.currentFrame) > 1
      ) {
        dotLottie.setFrame(appState.currentFrame);
      }
    }
  });

  // Handle file conversion
  let animationSrc = $derived.by(() => {
    if (!appState.currentFile?.content) return null;

    if (appState.currentFile.type === "lottie") {
      const blob = new Blob([appState.currentFile.content as any], {
        type: "application/zip",
      });
      return URL.createObjectURL(blob);
    }
    return null;
  });

  let animationData = $derived.by(() => {
    if (!appState.currentFile?.content) return null;

    if (appState.currentFile.type === "json") {
      try {
        const decoder = new TextDecoder();
        const jsonStr = decoder.decode(appState.currentFile.content);
        return JSON.parse(jsonStr);
      } catch (e) {
        console.error("Failed to parse JSON", e);
        return null;
      }
    }
    return null;
  });

  function onEvent(event: any) {
    if (event.name === "load" || event.name === "ready") {
      if (dotLottie) {
        // Extract metadata
        // Note: Some properties might need a small delay or check if loaded
        appState.totalFrames = dotLottie.totalFrames || 0;
        appState.duration = dotLottie.duration || 0;

        // Try to get dimensions if available (might depend on specific dotLottie version/implementation)
        // Often available via .animationSize or similar, but let's check what's available
        // For now, we can try to infer fps
        if (appState.duration > 0) {
          appState.fps = appState.totalFrames / appState.duration;
        }

        // If we can access the animation data directly to get w/h
        if (appState.currentFile?.type === "json" && animationData) {
          appState.originalWidth = animationData.w || 0;
          appState.originalHeight = animationData.h || 0;
        }
        // For .lottie, it's harder to get w/h without parsing the internal json,
        // but dotLottie might expose it.
      }
    }
    if (event.name === "frame") {
      // Update state only if we are playing to avoid loop with the seeker
      if (appState.isPlaying) {
        appState.currentFrame = event.data;
      }
    }
  }
  let { openFile } = $props<{ openFile: () => void }>();

  const onUpload: FileDropZoneProps["onUpload"] = async (files) => {
    appState.loadFromFile(files[0]);
  };

  const onFileRejected: FileDropZoneProps["onFileRejected"] = async ({
    reason,
    file,
  }) => {
    toast.error(`${file.name} failed to upload!`, { description: reason });
  };
</script>

<div
  class="w-full h-full flex items-center justify-center overflow-hidden relative bg-slate-900/50"
  style:background-color={appState.currentFile ? appState.backgroundColor : ""}
>
  {#if appState.currentFile}
    <div
      class:w-full={appState.scaleMode === "fit"}
      class:h-full={appState.scaleMode === "fit"}
      class="transition-all duration-300 ease-out"
      in:fade={{ duration: 300 }}
      style={appState.scaleMode === "original"
        ? `width: ${appState.originalWidth || 500}px; height: ${appState.originalHeight || 500}px`
        : ""}
    >
      {#if animationSrc}
        <DotLottieSvelte
          src={animationSrc}
          loop={appState.loop}
          autoplay={appState.isPlaying}
          speed={appState.speed}
          dotLottieRefCallback={(ref) => {
            dotLottie = ref;
            ref.addEventListener("load", onEvent);
            ref.addEventListener("frame", onEvent);
            ref.addEventListener("ready", onEvent);
          }}
          useFrameInterpolation={true}
        />
      {:else if animationData}
        <DotLottieSvelte
          data={animationData}
          loop={appState.loop}
          autoplay={appState.isPlaying}
          speed={appState.speed}
          dotLottieRefCallback={(ref) => {
            dotLottie = ref;
            ref.addEventListener("load", onEvent);
            ref.addEventListener("frame", onEvent);
            ref.addEventListener("ready", onEvent);
          }}
          useFrameInterpolation={true}
        />
      {/if}
    </div>
  {:else}
    <FileDropZone
      class="w-fit"
      {onUpload}
      {onFileRejected}
      accept="application/json, application/zip+dotlottie"
      maxFiles={1}
    />
  {/if}
</div>
