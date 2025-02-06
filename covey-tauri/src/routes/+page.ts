import { IconCache } from "$lib/icons";
import { Menu } from "$lib/menu.svelte";

import type { PageLoad } from "./$types";

// menu must be initialised in page load (not layout load),
// to avoid being re-initialised when settings page is opened.
export const load: PageLoad = async () => ({
  menu: await Menu.new(),
  iconCache: new IconCache(),
});
