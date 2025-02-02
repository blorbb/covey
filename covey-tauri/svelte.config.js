// Tauri doesn't have a Node.js server to do proper SSR
// so we will use adapter-static to prerender the app (SSG)
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
import adapter from "@sveltejs/adapter-static";
import autoprefixer from "autoprefixer";
import { sveltePreprocess } from "svelte-preprocess";
import { dirname, join } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: [
    sveltePreprocess({
      postcss: { plugins: [autoprefixer()] },
      scss: {
        prependData: `@use 'prelude.scss' as *;`,
        includePaths: [join(__dirname, "src")],
      },
    }),
  ],
  kit: {
    adapter: adapter(),
  },
};

export default config;
