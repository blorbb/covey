<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";

  import type { SchemaFilePath } from "$lib/bindings";
  import { type DeepReadonly, unreachable } from "$lib/utils";

  let {
    schema,
    userValue = $bindable(),
  }: { schema: DeepReadonly<SchemaFilePath>; userValue?: string } = $props();

  let error: undefined = $state();

  const pickFile = async () => {
    const filePath = await open({
      multiple: false,
      directory: false,
      filters:
        schema.extension == null
          ? undefined
          : [{ name: "Supported types", extensions: [...schema.extension] }],
    });

    if (filePath != null) {
      userValue = filePath;
    }
  };
</script>

<div class="input-file-path">
  <label>
    <button class="input-file-path-input" onclick={pickFile}>Open file</button>
    {userValue ?? schema.default ?? "No file selected"}
  </label>
  {#if error === undefined}{:else}
    {unreachable(error)}
  {/if}
</div>

<style lang="scss">
  .input-file-path-input {
    border: 2px solid var(--color-outline);
    background: var(--color-surface-container);
    padding: 0.25rem 0.5rem;
  }
</style>
