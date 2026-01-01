<script lang="ts">
  import Picker from "$lib/components/Picker.svelte";
  import { appState } from "$lib/state.svelte";
  import { DotLottieSvelte } from "@lottiefiles/dotlottie-svelte";
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

  function onEvent(event: any, player: any) {
    const type = event.type || event.name;
    if (type === "load" || type === "ready") {
      if (player) {
        // Extract metadata
        appState.totalFrames = player.totalFrames || 0;
        appState.duration = player.duration || 0;

        // Try to get dimensions
        if (appState.currentFile?.type === "json" && animationData) {
          appState.originalWidth = animationData.w || 0;
          appState.originalHeight = animationData.h || 0;
        } else if (player.view) {
          // Fallback to canvas dimensions
          appState.originalWidth = player.view.width || 0;
          appState.originalHeight = player.view.height || 0;
        }

        if (appState.duration > 0) {
          appState.fps = appState.totalFrames / appState.duration;
        } else if (appState.currentFile?.type === "json" && animationData) {
          appState.fps = animationData.fr || 30;
          if (appState.totalFrames > 0) {
            appState.duration = appState.totalFrames / appState.fps;
          }
        }
      }
    }
    if (type === "frame") {
      if (appState.isPlaying) {
        appState.currentFrame = event.currentFrame ?? event.data ?? 0;
      }
    }
  }
</script>

<div
  class="w-full h-full flex items-center justify-center overflow-hidden relative"
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
            const handler = (e: any) => onEvent(e, ref);
            ref.addEventListener("load", handler);
            ref.addEventListener("frame", handler);
            ref.addEventListener("ready", handler);
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
            const handler = (e: any) => onEvent(e, ref);
            ref.addEventListener("load", handler);
            ref.addEventListener("frame", handler);
            ref.addEventListener("ready", handler);
          }}
          useFrameInterpolation={true}
        />
      {/if}
    </div>
  {:else}
    <Picker />
  {/if}
</div>
