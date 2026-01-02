<script lang="ts">
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Label } from "$lib/components/ui/label";
  import * as Select from "$lib/components/ui/select";
  import { appConfig, type AppConfig } from "$lib/config.svelte";
  import TablerLanguage from "~icons/tabler/language";
  import TablerPalette from "~icons/tabler/palette";

  const languages: AppConfig["lang"][] = ["en", "ar"];

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
    <Card>
      <CardHeader>
        <div class="flex items-center gap-2">
          <TablerPalette class="text-muted-foreground" />
          <CardTitle>Appearance</CardTitle>
        </div>
      </CardHeader>
      <CardContent class="space-y-4">
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
      </CardContent>
    </Card>

    <!-- Language Selection -->
    <Card>
      <CardHeader>
        <div class="flex items-center gap-2">
          <TablerLanguage class="text-muted-foreground" />
          <CardTitle>Language</CardTitle>
        </div>
      </CardHeader>
      <CardContent class="space-y-4">
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
      </CardContent>
    </Card>
  </div>
</div>

<style>
  :global(body) {
    overflow: hidden;
  }
</style>
