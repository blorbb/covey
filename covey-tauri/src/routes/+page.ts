import { IconCache } from "$lib/icons";
import { Menu } from "$lib/menu.svelte";

import type { PageLoad } from "./$types";

export const load: PageLoad = async () => {
  return {
    menu: await Menu.new(),
    iconCache: new IconCache(),
  };
};
