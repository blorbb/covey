<script lang="ts">
  import type { JsonValue, SchemaType } from "$lib/bindings";
  import { type DeepReadonly, unreachable } from "$lib/utils";

  import InputBool from "./input_bool.svelte";
  import InputFilePath from "./input_file_path.svelte";
  import InputFolderPath from "./input_folder_path.svelte";
  import InputInt from "./input_int.svelte";
  import InputList from "./input_list.svelte";
  import InputMap from "./input_map.svelte";
  import InputSelection from "./input_selection.svelte";
  import InputStruct from "./input_struct.svelte";
  import InputText from "./input_text.svelte";

  let {
    schema,
    userValue = $bindable(),
    error = $bindable(),
    showStructLabels = true,
  }: {
    schema: DeepReadonly<SchemaType>;
    userValue?: JsonValue;
    error?: string;
    /** Whether to show labels if the input type is a struct. */
    showStructLabels?: boolean;
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
  const asArray = $derived(Array.isArray(userValue) ? userValue : undefined);
  const asMap = $derived(
    typeof userValue === "object" &&
      !Array.isArray(userValue) &&
      userValue != null
      ? userValue
      : undefined,
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
  <InputFolderPath
    schema={schema["folder-path"]}
    bind:userValue={() => asString, setUserValue}
    bind:error
  />
{:else if "selection" in schema}
  <InputSelection
    schema={schema.selection}
    bind:userValue={() => asString, setUserValue}
    bind:error
  />
{:else if "list" in schema}
  <InputList
    schema={schema.list}
    bind:userValue={() => asArray, setUserValue}
    bind:error
  />
{:else if "map" in schema}
  <InputMap
    schema={schema.map}
    bind:userValue={() => asMap, setUserValue}
    bind:error
  />
{:else if "struct" in schema}
  <InputStruct
    schema={schema.struct}
    bind:userValue={() => asMap, setUserValue}
    bind:error
    showLabels={showStructLabels}
  />
{:else}
  {unreachable(schema)}
{/if}
