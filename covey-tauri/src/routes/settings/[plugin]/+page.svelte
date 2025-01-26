<script lang="ts">
  import { page } from "$app/state";
  import Command from "$lib/components/command.svelte";
  import Divider from "$lib/components/divider.svelte";

  import type { PageData } from "../$types";

  let { data }: { data: PageData } = $props();
  const settings = data.settings;

  const pluginName = $derived(decodeURIComponent(page.params.plugin));
  const pluginCommands = $derived(
    settings.globalConfig.plugins[pluginName].commands,
  );

  const manifest = $derived(settings.manifestOf(pluginName));
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
    {#each Object.entries(manifest.commands) as [commandId, command] (commandId)}
      <Command
        {command}
        customHotkey={commandId in pluginCommands
          ? pluginCommands[commandId]
          : undefined}
        setCustomHotkey={(hotkey) => (pluginCommands[commandId] = hotkey)}
      />
    {/each}
  </div>

  {@const schema = Object.entries(manifest.schema)}
  {#if schema.length > 0}
    <Divider margin="1rem" />
    <h2>Configuration</h2>
    {#each schema as [_, config]}
      {config.title}
    {/each}
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
</style>
