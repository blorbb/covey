<script lang="ts">
  import type { JsonValue, SchemaType } from "$lib/bindings";
  import type { DeepReadonly } from "$lib/utils";

  import InputText from "./input_text.svelte";

  let {
    schema,
    userValue = $bindable(),
  }: { schema: DeepReadonly<SchemaType>; userValue?: JsonValue } = $props();

  const unreachable = (x: never): never => x;
</script>

{#if "text" in schema}
  <InputText
    schema={schema.text}
    bind:userValue={() =>
      typeof userValue === "string" ? userValue : undefined,
    (newValue) => (userValue = newValue)}
  />
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
