<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";

  import type { SchemaFilePath } from "$lib/bindings";
  import type { DeepReadonly } from "$lib/utils";

  let {
    schema,
    userValue = $bindable(),
    error = $bindable(),
  }: {
    schema: DeepReadonly<SchemaFilePath>;
    userValue?: string;
    error?: string;
  } = $props();

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
      setValue(filePath);
    }
  };

  const setValue = (path: string) => {
    if (
      schema.extension == null ||
      schema.extension.some((ext) => path.endsWith(ext))
    ) {
      error = undefined;
      userValue = path;
    } else {
      error = `File extension must be one of: ${schema.extension.map((ext) => "." + ext).join(", ")}`;
    }
  };
</script>

<div class="input-file-path-wrapper">
  <input
    type="text"
    class="input-file-path"
    value={userValue ?? schema.default ?? ""}
    onchange={(e) => setValue(e.currentTarget.value)}
    onfocusout={(e) =>
      (e.currentTarget.scrollLeft = e.currentTarget.scrollWidth)}
  />
  <button class="input-file-path-picker" onclick={pickFile}> Pick file </button>
</div>

<style lang="scss">
  .input-file-path-wrapper {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 0.5rem;
  }

  .input-file-path,
  .input-file-path-picker {
    border: 2px solid var(--color-outline);
    background: var(--color-surface-container);
    padding: 0.25rem 0.5rem;
  }

  .input-file-path-picker {
    border-radius: 999rem;
  }
</style>
