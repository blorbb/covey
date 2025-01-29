<script lang="ts">
  import type { SchemaType } from "$lib/bindings";
  import type { DeepReadonly } from "$lib/utils";

  import InputStr from "./input_text.svelte";

  let { schema }: { schema: DeepReadonly<SchemaType> } = $props();

  const unreachable = (x: never): never => x;
</script>

{#if "text" in schema}
  <InputStr schema={schema.text} />
{:else if "int" in schema}
  todo int
{:else if "file-path" in schema}
  todo file path
{:else if "folder-path" in schema}
  todo folder path
{:else if "bool" in schema}
  todo bool
{:else if "list" in schema}
  todo list
{:else if "map" in schema}
  todo map
{:else if "struct" in schema}
  todo struct
{:else}
  {unreachable(schema)}
{/if}
