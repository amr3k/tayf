<script lang="ts">
  import { Toaster } from "$lib/components/ui/sonner/index.js";
  import "@fontsource-variable/inter";
  import "@fontsource-variable/rubik";
  import "../app.css";

  let { children } = $props();
</script>

<Toaster position="bottom-left" />
{@render children()}
