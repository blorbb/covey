import { IconCache } from "$lib/icons";
import { Menu } from "$lib/menu.svelte";
import { Settings } from "$lib/settings.svelte";

import type { PageLoad } from "./$types";

// menu must be initialised in page load (not layout load),
// to avoid being re-initialised when settings page is opened.
// settings must also be initialised after the menu, so it can't
// be put in layout.
export const load: PageLoad = async () => ({
  menu: await Menu.new(),
  iconCache: new IconCache(),
  settings: await Settings.new(),
});
