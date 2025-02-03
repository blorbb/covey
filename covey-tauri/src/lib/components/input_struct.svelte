<script lang="ts">
  import type { JsonValue, SchemaStruct } from "$lib/bindings";
  import type { DeepReadonly } from "$lib/utils";

  import InputField from "./input_field.svelte";

  let {
    schema,
    userValue = $bindable(),
    error = $bindable(),
    showLabels,
  }: {
    schema: DeepReadonly<SchemaStruct>;
    userValue?: Record<string, JsonValue>;
    error?: string;
    showLabels: boolean;
  } = $props();

  let drafts: [string, JsonValue | undefined][] = $state(
    // get all fields defined in schema
    Object.entries(schema.fields).map(([field]) => [field, userValue?.[field]]),
  );

  // error for each of the values
  // must be kept the same length as `drafts`
  let errors = $state<(string | undefined)[]>(drafts.map(() => undefined));

  $effect(() => {
    // set the shown error to the first one
    error = errors.find((err) => err != null);
  });

  $effect(() => {
    if (drafts.every((draft) => draft[1] != null)) {
      userValue = Object.fromEntries(drafts as [string, JsonValue][]);
    }
  });
</script>

<div class="input-struct">
  {#if showLabels}
    <div class="input-struct-labels">
      {#each Object.keys(schema.fields) as field (field)}
        <div class="input-struct-label">
          {field}
        </div>
      {/each}
    </div>
  {/if}

  <div class="input-struct-inputs">
    {#each drafts as _, i}
      <div class="input-struct-item">
        <InputField
          schema={schema.fields[drafts[i][0]]}
          bind:userValue={drafts[i][1]}
          bind:error={errors[i]}
        />
      </div>
    {/each}
  </div>
</div>

<style lang="scss">
  .input-struct-labels,
  .input-struct-inputs {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(0, 1fr));
  }

  .input-struct-labels {
    text-align: center;
    font-size: var(--fs-small);
    font-weight: bold;
  }

  .input-struct-label {
    background: var(--color-surface-container);
    border-bottom: 2px solid var(--color-tertiary);
  }
</style>
