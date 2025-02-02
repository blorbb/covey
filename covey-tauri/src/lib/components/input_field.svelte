<script lang="ts">
  import type { JsonValue, SchemaType } from "$lib/bindings";
  import { type DeepReadonly, unreachable } from "$lib/utils";

  import InputBool from "./input_bool.svelte";
  import InputFilePath from "./input_file_path.svelte";
  import InputInt from "./input_int.svelte";
  import InputText from "./input_text.svelte";

  let {
    schema,
    userValue = $bindable(),
    error = $bindable(),
  }: {
    schema: DeepReadonly<SchemaType>;
    userValue?: JsonValue;
    error?: string;
  } = $props();

  const asNumber = $derived(
    typeof userValue === "number" ? userValue : undefined,
  );
  const asString = $derived(
    typeof userValue === "string" ? userValue : undefined,
  );
  const asBool = $derived(
    typeof userValue === "boolean" ? userValue : undefined,
  );

  const setUserValue = (value: JsonValue | undefined): void => {
    userValue = value;
  };
</script>

{#if "text" in schema}
  <InputText
    schema={schema.text}
    bind:userValue={() => asString, setUserValue}
    bind:error
  />
{:else if "int" in schema}
  <InputInt
    schema={schema.int}
    bind:userValue={() => asNumber, setUserValue}
    bind:error
  />
{:else if "bool" in schema}
  <InputBool
    schema={schema.bool}
    bind:userValue={() => asBool, setUserValue}
    bind:error
  />
{:else if "file-path" in schema}
  <InputFilePath
    schema={schema["file-path"]}
    bind:userValue={() => asString, setUserValue}
    bind:error
  />
{:else if "folder-path" in schema}
  todo folder path
{:else if "list" in schema}
  todo list
{:else if "map" in schema}
  todo map
{:else if "struct" in schema}
  todo struct
{:else}
  {unreachable(schema)}
{/if}
