// Tauri doesn't have a Node.js server to do proper SSR
// so we will use adapter-static to prerender the app (SSG)

import { IconCache } from "$lib/icons";
import { Menu } from "$lib/menu.svelte";
import type { LayoutLoad } from "./$types";

// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
export const prerender = true;
export const ssr = false;

export const load: LayoutLoad = async () => {
  return {
    menu: await Menu.new(),
    iconCache: new IconCache(),
  };
};
