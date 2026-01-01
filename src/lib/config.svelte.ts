import { invoke } from "@tauri-apps/api/core";
import { setMode } from "mode-watcher";

export interface AppConfig {
  theme: "system" | "light" | "dark";
}

class ConfigService {
  #config = $state<AppConfig>({ theme: "system" });
  #initialized = $state(false);

  constructor() {
    this.init();
  }

  get theme() {
    return this.#config.theme;
  }

  set theme(value: AppConfig["theme"]) {
    this.#config.theme = value;
    this.save();
    setMode(value);
  }

  get isInitialized() {
    return this.#initialized;
  }

  async init() {
    try {
      const config = await invoke<AppConfig>("get_config");
      this.#config = config;
      setMode(config.theme);
      this.#initialized = true;
    } catch (e) {
      console.error("Failed to load config:", e);
      this.#initialized = true;
    }
  }

  async save() {
    try {
      await invoke("set_config", { config: { theme: this.#config.theme } });
    } catch (e) {
      console.error("Failed to save config:", e);
    }
  }
}

export const appConfig = new ConfigService();
