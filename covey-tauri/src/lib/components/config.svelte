<script lang="ts">
  import type { JsonValue, PluginConfigSchema } from "$lib/bindings";
  import type { DeepReadonly } from "$lib/utils";

  import InputField from "./input_field.svelte";

  let {
    schema,
    userValue = $bindable(),
  }: { schema: DeepReadonly<PluginConfigSchema>; userValue?: JsonValue } =
    $props();

  let error = $state<string>();
</script>

<div class="config">
  <p class="config-title">{schema.title}</p>
  {#if schema.description != null}
    <p class="config-description">
      {schema.description}
    </p>
  {/if}
  <div class="config-input">
    <InputField schema={schema.type} bind:userValue bind:error />
  </div>
  {#if error != null}
    <p class="config-error">
      Error: {error}
    </p>
  {/if}
</div>

<style lang="scss">
  .config {
    display: grid;
    grid-template-areas:
      "title"
      "description"
      "input"
      "error";
    grid-template-columns: 1fr fit-content(50%);
  }

  .config-title {
    grid-area: title;
    font-weight: bold;
  }

  .config-description {
    grid-area: description;
    font-size: var(--fs-small);
    color: var(--color-on-surface-variant);
    // allow newline characters to make a new line
    white-space: pre-line;
  }

  .config-input {
    grid-area: input;
    width: 100%;
  }

  .config-error {
    grid-area: error;
    color: var(--color-error);
  }
</style>
