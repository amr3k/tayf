<script lang="ts">
  import * as Card from "$lib/components/ui/card";
  import { Label } from "$lib/components/ui/label";
  import * as Select from "$lib/components/ui/select";
  import { appConfig, type AppConfig } from "$lib/config.svelte";
  import { languages } from "$lib/languages";
  import TablerLanguage from "~icons/tabler/language";
  import TablerPalette from "~icons/tabler/palette";

  const getLanguageLabel = (code: string) => {
    try {
      return (
        new Intl.DisplayNames([code], { type: "language" }).of(code) || code
      );
    } catch {
      return "English";
    }
  };
</script>

<div class="min-h-screen bg-background p-6 space-y-6">
  <div class="flex items-center gap-2 mb-8">
    <h1 class="text-2xl font-bold tracking-tight">Preferences</h1>
  </div>

  <div class="grid gap-6">
    <!-- Theme Selection -->
    <Card.Root>
      <Card.Header>
        <div class="flex items-center gap-2">
          <TablerPalette class="text-muted-foreground" />
          <Card.Title>Appearance</Card.Title>
        </div>
      </Card.Header>
      <Card.Content class="space-y-4">
        <div class="space-y-2">
          <Label>Theme</Label>
          <Select.Root
            type="single"
            value={appConfig.theme}
            onValueChange={(v) => {
              if (v) appConfig.theme = v as AppConfig["theme"];
            }}
          >
            <Select.Trigger class="w-full">
              {#if appConfig.theme === "system"}
                System
              {:else if appConfig.theme === "light"}
                Light
              {:else}
                Dark
              {/if}
            </Select.Trigger>
            <Select.Content>
              <Select.Item value="system" label="System">System</Select.Item>
              <Select.Item value="light" label="Light">Light</Select.Item>
              <Select.Item value="dark" label="Dark">Dark</Select.Item>
            </Select.Content>
          </Select.Root>
        </div>
      </Card.Content>
    </Card.Root>

    <!-- Language Selection -->
    <Card.Root>
      <div class="flex items-center gap-2">
        <TablerLanguage class="text-muted-foreground" />
        <Card.Title>Language</Card.Title>
      </div>
      <Card.Content class="space-y-4">
        <div class="space-y-2">
          <Label>System Language</Label>
          <Select.Root
            type="single"
            value={appConfig.lang}
            onValueChange={(v) => {
              if (v) appConfig.lang = v as any;
            }}
          >
            <Select.Trigger class="w-full">
              {getLanguageLabel(appConfig.lang)}
            </Select.Trigger>
            <Select.Content>
              {#each languages as lang}
                <Select.Item value={lang} label={getLanguageLabel(lang)}>
                  {getLanguageLabel(lang)}
                </Select.Item>
              {/each}
            </Select.Content>
          </Select.Root>
        </div>
      </Card.Content>
    </Card.Root>
  </div>
</div>

<style>
  :global(body) {
    overflow: hidden;
  }
</style>
