<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let name = $state("");
  let greetMsg = $state("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    greetMsg = await invoke("greet", { name });
  }
</script>

<div class="container">
  <h1>Welcome to Tauri!</h1>

  <input placeholder="Enter a name..." bind:value={name} />
  <button onclick={greet}>Greet</button>

  <p>{greetMsg}</p>
</div>
