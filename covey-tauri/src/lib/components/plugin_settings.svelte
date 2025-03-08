<script lang="ts">
  import type { PluginConfig, PluginManifest } from "$lib/bindings";
  import type { DeepReadonly } from "$lib/utils";

  import Command from "./command.svelte";
  import Config from "./config.svelte";
  import Divider from "./divider.svelte";

  let {
    plugin = $bindable(),
    manifest,
  }: { plugin: PluginConfig; manifest: DeepReadonly<PluginManifest> } =
    $props();
</script>

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

<Config
  schema={{
    title: "Enable plugin",
    description:
      "A prefix must also be defined, otherwise the plugin will stay disabled.",
    id: "enabled",
    type: {
      bool: {
        default: false,
      },
    },
  }}
  bind:userValue={() => !plugin.disabled,
  // typescript can't figure out that this is a boolean for some reason
  (enabled) => (plugin.disabled = !(enabled as boolean))}
/>

<Config
  schema={{
    title: "Prefix",
    description:
      "Prefix to activate this plugin. Note that this is whitespace sensitive.",
    id: "prefix",
    type: {
      text: {
        "min-length": 0,
        "max-length": Number.MAX_SAFE_INTEGER,
        default: manifest["default-prefix"],
      },
    },
  }}
  bind:userValue={plugin.prefix}
/>

<Divider margin="1rem" />

<h2>Commands</h2>
<div class="commands">
  <!-- command is not reactive, so need to remount if the plugin changes -->
  {#each manifest.commands as command (`${plugin.id} : ${command.id}`)}
    <Command
      {command}
      bind:userHotkeys={() => plugin.commands[command.id]?.hotkeys ?? undefined,
      (hotkeys) => (plugin.commands[command.id] = { hotkeys: hotkeys ?? null })}
    />
  {/each}
</div>

{#if manifest.schema.length > 0}
  <Divider margin="1rem" />
  <h2>Configuration</h2>
  <div class="configs">
    {#each manifest.schema as schema (`${plugin.id} : ${schema.id}`)}
      <Config {schema} bind:userValue={plugin.settings[schema.id]} />
    {/each}
  </div>
{/if}

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
