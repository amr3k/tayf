import { adapter as svelte } from "@wuchale/svelte";
import { defineConfig } from "wuchale";
import { adapter as js } from "wuchale/adapter-vanilla";
import languages from "./languages.json" with { type: "json" };

export default defineConfig({
  locales: languages,
  adapters: {
    main: svelte({ loader: "sveltekit" }),
    js: js({
      loader: "vite",
      files: [
        "src/**/+{page,layout}.{js,ts}",
        "src/**/+{page,layout}.server.{js,ts}",
        "src/**/*.svelte.{js,ts}",
      ],
    }),
  },
});
