<script lang="ts">
  import Picker from "$lib/components/Picker.svelte";
  import { appState } from "$lib/state.svelte";
  import { DotLottieSvelte } from "@lottiefiles/dotlottie-svelte";
  import { onDestroy } from "svelte";

  let dotLottie: any = $state(null);
  let eventHandlers: Array<(e: any) => void> = [];
  let blobUrl = $state<string | null>(null);
  let prevFileContent = $state<Uint8Array | null>(null);

  $effect(() => {
    return () => {
      if (dotLottie && eventHandlers.length > 0) {
        eventHandlers.forEach((handler) => {
          dotLottie.removeEventListener("load", handler);
          dotLottie.removeEventListener("frame", handler);
          dotLottie.removeEventListener("ready", handler);
        });
        eventHandlers = [];
      }

      if (blobUrl) {
        URL.revokeObjectURL(blobUrl);
        blobUrl = null;
      }
    };
  });

  $effect(() => {
    if (!dotLottie) return;
    if (appState.isPlaying) {
      dotLottie.play();
    } else {
      dotLottie.pause();
    }
  });

  $effect(() => {
    if (!dotLottie || appState.isPlaying || !dotLottie.isLoaded) return;
    dotLottie.setFrame(appState.currentFrame);
  });

  $effect(() => {
    const content = appState.currentFile?.content;
    const type = appState.currentFile?.type;

    if (!content || type !== "lottie") {
      if (blobUrl) {
        URL.revokeObjectURL(blobUrl);
        blobUrl = null;
      }
      prevFileContent = null;
      return;
    }

    if (prevFileContent === content) return;

    if (blobUrl) {
      URL.revokeObjectURL(blobUrl);
    }

    const blob = new Blob([content as BlobPart], {
      type: "application/zip",
    });
    blobUrl = URL.createObjectURL(blob);
    prevFileContent = content;
  });

  let animationSrc = $derived.by(() => {
    if (appState.currentFile?.type === "lottie") {
      return blobUrl;
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
        appState.error = "Invalid Lottie JSON file";
        return null;
      }
    }
    return null;
  });

  function onEvent(event: any, player: any) {
    const type = event.type || event.name;
    if (type === "load" || type === "ready") {
      if (player) {
        appState.totalFrames = player.totalFrames || 0;
        appState.duration = player.duration || 0;

        if (appState.currentFile?.type === "json" && animationData) {
          appState.originalWidth = (animationData as any).w || 0;
          appState.originalHeight = (animationData as any).h || 0;
        } else if (player.view) {
          appState.originalWidth = player.view.width || 0;
          appState.originalHeight = player.view.height || 0;
        }

        if (appState.duration > 0) {
          appState.fps = appState.totalFrames / appState.duration;
        } else if (appState.currentFile?.type === "json" && animationData) {
          appState.fps = (animationData as any).fr || 30;
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

  onDestroy(() => {
    if (dotLottie) {
      dotLottie.destroy();
    }
  })
</script>

  <div
    class="w-full h-full flex items-center justify-center overflow-hidden relative"
    style:background-color={appState.currentFile ? appState.backgroundColor : ""}
  >
    {#if appState.currentFile}
      <div
        class="w-full h-full flex items-center justify-center p-4"
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
                eventHandlers.push(handler);
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
                eventHandlers.push(handler);
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
