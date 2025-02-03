<script lang="ts">
  import { page } from "$app/state";
  import type { PluginConfig } from "$lib/bindings";
  import Command from "$lib/components/command.svelte";
  import Config from "$lib/components/config.svelte";
  import Divider from "$lib/components/divider.svelte";

  import type { PageData } from "../$types";

  let { data }: { data: PageData } = $props();
  const settings = data.settings;

  const pluginId = $derived(decodeURIComponent(page.params.plugin));
  // TODO: handle invalid id
  const plugin = $derived(settings.getPlugin(pluginId)) as PluginConfig;

  const manifest = $derived(settings.fetchManifestOf(pluginId));

  $effect(() => {
    settings.updateBackendConfig();
  });
</script>

{#await manifest then manifest}
  <h1 class="plugin-title">{manifest.name}</h1>

  {#if manifest.description != null}
    <p class="description">
      {manifest.description}
    </p>
  {/if}
  {#if manifest.authors.length > 0}
    <p class="authors">
      By
      {manifest.authors.join(", ")}
    </p>
  {/if}
  {#if manifest.repository != null}
    <p class="repo">
      Repository:
      <a href={manifest.repository} target="_blank">
        {manifest.repository}
      </a>
    </p>
  {/if}

  <Divider margin="1rem" />

  <h2>Commands</h2>
  <div class="commands">
    {#each manifest.commands as command (command.id)}
      <Command {command} bind:userHotkey={plugin.commands[command.id]} />
    {/each}
  </div>

  {#if manifest.schema.length > 0}
    <Divider margin="1rem" />
    <h2>Configuration</h2>
    <div class="configs">
      {#each manifest.schema as schema}
        <Config {schema} bind:userValue={plugin.config[schema.id]} />
      {/each}
    </div>
  {/if}
{/await}

<style lang="scss">
  .plugin-title {
    line-height: 2;
  }

  .description,
  .authors,
  .repo {
    color: var(--color-on-surface-variant);
    font-size: var(--fs-small);
  }

  .commands {
    display: grid;
    gap: 1rem;
  }

  .configs {
    display: grid;
    gap: 1rem;
  }
</style>
