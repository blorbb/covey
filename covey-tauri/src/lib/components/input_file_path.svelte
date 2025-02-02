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

  error = undefined;

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

  $inspect("in file path", userValue);
</script>

<button class="input-file-path" onclick={pickFile}>
  <!-- needs to be inside a div to show ellipsis correctly -->
  <div>{userValue ?? schema.default ?? "Pick file"}</div>
</button>

<style lang="scss">
  .input-file-path {
    border: 2px solid var(--color-outline);
    background: var(--color-surface-container);
    padding: 0.25rem 0.5rem;
    text-overflow: ellipsis;

    width: 100%;

    > div {
      overflow: hidden;
      white-space: nowrap;
      text-overflow: ellipsis;
    }
  }
</style>
