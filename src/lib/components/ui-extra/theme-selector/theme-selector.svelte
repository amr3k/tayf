<script lang="ts" module>
  export type ThemeSelectorProps = {
    variant?: "outline" | "ghost";
  };
</script>

<script lang="ts">
  import * as DropdownMenu from "$lib/components/ui-extra/dropdown-menu";
  import { buttonVariants } from "$lib/components/ui/button/index.js";
  import { appConfig } from "$lib/config.svelte";
  import MoonIcon from "@lucide/svelte/icons/moon";
  import SunIcon from "@lucide/svelte/icons/sun";

  let { variant = "outline" }: ThemeSelectorProps = $props();
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger class={buttonVariants({ variant, size: "icon" })}>
    <SunIcon
      class="scale-100 rotate-0 !transition-all dark:scale-0 dark:-rotate-90"
    />
    <MoonIcon
      class="absolute scale-0 rotate-90 !transition-all dark:scale-100 dark:rotate-0"
    />
    <span class="sr-only">Toggle theme</span>
  </DropdownMenu.Trigger>
  <DropdownMenu.Content align="end">
    <DropdownMenu.Item onclick={() => (appConfig.theme = "light")}
      >Light</DropdownMenu.Item
    >
    <DropdownMenu.Item onclick={() => (appConfig.theme = "dark")}
      >Dark</DropdownMenu.Item
    >
    <DropdownMenu.Item onclick={() => (appConfig.theme = "system")}
      >System</DropdownMenu.Item
    >
  </DropdownMenu.Content>
</DropdownMenu.Root>
