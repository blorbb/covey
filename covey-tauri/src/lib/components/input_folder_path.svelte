<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";

  import type { SchemaFolderPath } from "$lib/bindings";
  import type { DeepReadonly } from "$lib/utils";

  import InputPath from "./input_path.svelte";

  let {
    schema,
    userValue = $bindable(),
    error = $bindable(),
  }: {
    schema: DeepReadonly<SchemaFolderPath>;
    userValue?: string;
    error?: string;
  } = $props();

  const pickFile = async () => {
    const folderPath = await open({
      multiple: false,
      directory: true,
    });

    if (folderPath != null) {
      setValue(folderPath);
    }
  };

  const setValue = (path: string) => {
    error = undefined;
    userValue = path;
  };
</script>

<InputPath
  value={userValue ?? schema.default ?? ""}
  onSetValue={setValue}
  onPick={pickFile}
  buttonText="Pick folder"
/>
