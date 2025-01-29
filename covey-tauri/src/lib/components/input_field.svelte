<script lang="ts">
  import type { ConfigType } from "$lib/bindings";
  import type { DeepReadonly } from "$lib/utils";

  import InputStr from "./input_str.svelte";

  let { config }: { config: DeepReadonly<ConfigType> } = $props();

  const unreachable = (x: never): never => x;
</script>

{#if "str" in config}
  <InputStr config={config.str} />
{:else if "int" in config}
  todo int
{:else if "file-path" in config}
  todo file path
{:else if "folder-path" in config}
  todo folder path
{:else if "bool" in config}
  todo bool
{:else if "list" in config}
  todo list
{:else if "map" in config}
  todo map
{:else if "struct" in config}
  todo struct
{:else}
  {unreachable(config)}
{/if}
