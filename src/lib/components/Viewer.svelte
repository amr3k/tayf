<script lang="ts">
  import Picker from "$lib/components/Picker.svelte";
  import { appState } from "$lib/state.svelte";
  import { DotLottieSvelte } from "@lottiefiles/dotlottie-svelte";
  import { untrack } from "svelte";
  import { fade } from "svelte/transition";

  let dotLottie: any = $state(null);

  $effect(() => {
    if (!dotLottie) return;
    if (appState.isPlaying) {
      dotLottie.play();
    } else {
      dotLottie.pause();
    }
  });

  $effect(() => {
    if (!dotLottie) return;
    dotLottie.setSpeed(appState.speed);
    dotLottie.setLoop(appState.loop);
  });

  $effect(() => {
    if (!dotLottie) return;
    const frame = appState.currentFrame;
    // Only seek manually if we are paused (scrubbing)
    // While playing, the player handles its own frame progression
    if (!appState.isPlaying) {
      untrack(() => {
        dotLottie.setFrame(frame);
      });
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
      class="w-full h-full flex items-center justify-center p-4 transition-all duration-300 ease-out"
      in:fade={{ duration: 300 }}
    >
      {#key appState.currentFile}
        <div
          class="animation-container w-full h-full flex items-center justify-center"
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
              layout={{
                fit: "contain",
                align: [0.5, 0.5],
              }}
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
              layout={{
                fit: "contain",
                align: [0.5, 0.5],
              }}
            />
          {/if}
        </div>
      {/key}
    </div>
  {:else}
    <Picker />
  {/if}
</div>

<style>
  .animation-container :global(canvas),
  .animation-container :global(svg),
  .animation-container :global(dotlottie-player),
  .animation-container :global(iframe) {
    width: 100% !important;
    height: 100% !important;
    max-width: 100%;
    max-height: 100%;
    object-fit: contain !important;
  }
</style>
