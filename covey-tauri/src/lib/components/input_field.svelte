<script lang="ts">
  import type { JsonValue, SchemaType } from "$lib/bindings";
  import { type DeepReadonly, unreachable } from "$lib/utils";

  import InputFilePath from "./input_file_path.svelte";
  import InputInt from "./input_int.svelte";
  import InputText from "./input_text.svelte";

  let {
    schema,
    userValue = $bindable(),
  }: { schema: DeepReadonly<SchemaType>; userValue?: JsonValue } = $props();

  const getNumber = (x: unknown) => () =>
    typeof x === "number" ? x : undefined;
  const getString = (x: unknown) => () =>
    typeof x === "string" ? x : undefined;

  const setUserValue = (value: JsonValue | undefined): void => {
    userValue = value;
  };
</script>

{#if "text" in schema}
  <InputText
    schema={schema.text}
    bind:userValue={getString(userValue), setUserValue}
  />
{:else if "int" in schema}
  <InputInt
    schema={schema.int}
    bind:userValue={getNumber(userValue), setUserValue}
  />
{:else if "file-path" in schema}
  <InputFilePath
    schema={schema["file-path"]}
    bind:userValue={getString(userValue), setUserValue}
  />
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
