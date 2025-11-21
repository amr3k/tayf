<script lang="ts">
  import { DotLottieSvelte } from "@lottiefiles/dotlottie-svelte";
  import { appState } from "$lib/state.svelte";
  import { fade, fly } from "svelte/transition";

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

  let isDragging = $state(false);

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    isDragging = true;
  }

  function handleDragLeave(e: DragEvent) {
    e.preventDefault();
    isDragging = false;
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDragging = false;

    if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
      appState.loadFromFile(e.dataTransfer.files[0]);
    }
  }
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
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="flex flex-col items-center justify-center text-slate-500 gap-6 p-12 border-4 border-dashed rounded-3xl bg-slate-800/30 backdrop-blur-sm transition-all cursor-pointer hover:bg-slate-800/50 hover:border-blue-500/50 hover:text-blue-400"
      class:border-blue-500={isDragging}
      class:bg-blue-500_10={isDragging}
      class:text-blue-400={isDragging}
      class:border-slate-700_50={!isDragging}
      in:fly={{ y: 20, duration: 400 }}
      onclick={openFile}
      ondragover={handleDragOver}
      ondragleave={handleDragLeave}
      ondrop={handleDrop}
    >
      <div
        class="w-24 h-24 rounded-2xl bg-slate-800 flex items-center justify-center shadow-xl shadow-black/20 transition-transform group-hover:scale-110"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="48"
          height="48"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="text-blue-500"
          ><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline
            points="17 8 12 3 7 8"
          ></polyline><line x1="12" y1="3" x2="12" y2="15"></line></svg
        >
      </div>
      <div class="text-center space-y-2">
        <h3 class="text-2xl font-bold text-slate-200">Open Lottie File</h3>
        <p class="text-slate-400 max-w-xs">
          Drag and drop your Lottie JSON or .lottie files here to preview
        </p>
      </div>
    </div>
  {/if}
</div>
