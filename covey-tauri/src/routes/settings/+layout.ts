import { Settings } from "$lib/settings.svelte";

import type { LayoutLoad } from "../$types";

export const load: LayoutLoad = async () => {
  return {
    settings: await Settings.new(),
  };
};
