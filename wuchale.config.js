// @ts-check
import { adapter as svelte } from "@wuchale/svelte"
import { defineConfig } from "wuchale"
import { adapter as js } from 'wuchale/adapter-vanilla'

export default defineConfig({
  locales: ['en', 'ar'],
  adapters: {
    main: svelte({ loader: 'sveltekit' }),
    js: js({
      loader: 'vite',
      files: [
        'src/**/+{page,layout}.{js,ts}',
        'src/**/+{page,layout}.server.{js,ts}',
        'src/**/*.svelte.{js,ts}',
      ],
    })
  }
})