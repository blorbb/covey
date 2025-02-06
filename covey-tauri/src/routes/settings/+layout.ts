import { Settings } from "$lib/settings.svelte";

import type { LayoutLoad } from "./$types";

// menu must be initialised in page load (not layout load),
// to avoid being re-initialised when settings page is opened.
export const load: LayoutLoad = async () => ({
  settings: await Settings.new(),
});
