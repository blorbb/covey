<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";

  import type { SchemaFilePath } from "$lib/bindings";
  import type { DeepReadonly } from "$lib/utils";

  import InputPath from "./input_path.svelte";

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

<InputPath
  value={userValue ?? schema.default ?? ""}
  onSetValue={setValue}
  onPick={pickFile}
  buttonText="Pick file"
/>
