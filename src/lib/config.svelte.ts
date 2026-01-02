import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { setMode } from "mode-watcher";
import { type Locale } from "./languages";

export interface AppConfig {
  theme: "system" | "light" | "dark";
  lang: Locale;
}

class ConfigService {
  #config = $state<AppConfig>({ theme: "system", lang: "en" });
  #initialized = $state(false);
  #saveTimeout: ReturnType<typeof setTimeout> | null = null;

  constructor() {
    this.init();
  }

  get theme() {
    return this.#config.theme;
  }

  set theme(value: AppConfig["theme"]) {
    this.#config.theme = value;
    this.#debouncedSave();
    setMode(value);
  }

  get lang() {
    return this.#config.lang;
  }

  set lang(value: AppConfig["lang"]) {
    this.#config.lang = value;
    this.#debouncedSave();
  }

  get isInitialized() {
    return this.#initialized;
  }

  #debouncedSave() {
    if (this.#saveTimeout) {
      clearTimeout(this.#saveTimeout);
    }
    this.#saveTimeout = setTimeout(() => {
      this.save();
    }, 300);
  }

  async init() {
    try {
      const config = await invoke<AppConfig>("get_config");
      this.#config = config;
      setMode(config.theme);

      listen<AppConfig>("config-updated", (event) => {
        this.#config = event.payload;
        setMode(event.payload.theme);
      });

      this.#initialized = true;
    } catch (e) {
      console.error("Failed to load config:", e);
      this.#initialized = true;
    }
  }

  async save() {
    try {
      await invoke("set_config", { config: $state.snapshot(this.#config) });
    } catch (e) {
      console.error("Failed to save config:", e);
    }
  }
}

export const appConfig = new ConfigService();
