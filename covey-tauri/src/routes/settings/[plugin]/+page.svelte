<script lang="ts">
  import { page } from "$app/state";
  import type { PluginConfig } from "$lib/bindings";
  import PluginSettings from "$lib/components/plugin_settings.svelte";

  import type { LayoutData } from "./$types";

  let { data }: { data: LayoutData } = $props();
  const settings = data.settings;

  const pluginId = $derived(decodeURIComponent(page.params.plugin));
  // TODO: handle invalid id
  const plugin = $derived(settings.getPluginConfig(pluginId)) as PluginConfig;

  const manifest = $derived(settings.manifests[pluginId]);
</script>

<PluginSettings
  bind:plugin={() => plugin, (v) => settings.setPluginConfig(pluginId, v)}
  {manifest}
/>
