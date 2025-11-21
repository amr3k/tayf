<script lang="ts">
  import { DotLottieSvelte } from "@lottiefiles/dotlottie-svelte";
  import { appState } from "$lib/state.svelte";

  let dotLottie: any = $state(null);

  $effect(() => {
    if (dotLottie) {
      if (appState.isPlaying) {
        dotLottie.play();
      } else {
        dotLottie.pause();
      }
    }
  });

  $effect(() => {
    if (dotLottie) {
      dotLottie.setSpeed(appState.speed);
    }
  });

  $effect(() => {
    if (dotLottie) {
      // loop is handled via prop, but we can also set it imperatively if needed
      // dotLottie.setLoop(appState.loop);
    }
  });

  // Handle file conversion
  let animationSrc = $derived.by(() => {
    if (!appState.currentFile?.content) return null;

    if (appState.currentFile.type === "lottie") {
      const blob = new Blob([appState.currentFile.content], {
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
    if (event.name === "load") {
      // Initialize metadata if possible
      // Note: dotLottie instance might not be fully ready with metadata immediately
    }
    if (event.name === "frame") {
      appState.currentFrame = event.data;
    }
  }
</script>

<div
  class="w-full h-full flex items-center justify-center overflow-hidden"
  style:background-color={appState.backgroundColor}
>
  {#if appState.currentFile}
    <div
      class:w-full={appState.scaleMode === "fit"}
      class:h-full={appState.scaleMode === "fit"}
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
          bind:dotLottie
          useFrameInterpolation={true}
        />
      {:else if animationData}
        <DotLottieSvelte
          data={animationData}
          loop={appState.loop}
          autoplay={appState.isPlaying}
          speed={appState.speed}
          bind:dotLottie
          useFrameInterpolation={true}
        />
      {/if}
    </div>
  {:else}
    <div class="text-gray-400 flex flex-col items-center gap-4">
      <p class="text-lg">Open a Lottie file to start</p>
      <p class="text-sm opacity-60">Supports .json and .lottie</p>
    </div>
  {/if}
</div>
