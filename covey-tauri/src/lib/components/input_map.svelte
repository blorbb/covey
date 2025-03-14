<script lang="ts">
  import type { JsonValue, SchemaMap } from "$lib/bindings";
  import type { DeepReadonly } from "$lib/utils";

  import Button from "./button.svelte";
  import InputField from "./input_field.svelte";
  import InputText from "./input_text.svelte";

  let {
    schema,
    userValue = $bindable(),
    error = $bindable(),
  }: {
    schema: DeepReadonly<SchemaMap>;
    userValue?: Record<string, JsonValue>;
    error?: string;
  } = $props();

  let drafts: [string | undefined, JsonValue | undefined][] = $state(
    userValue == null ? [] : Object.entries(userValue),
  );

  // error for each of the values
  // must be kept the same length as `drafts`
  let errors = $state<(string | undefined)[]>(drafts.map(() => undefined));

  // index that contains a duplicate
  let duplicate_key_index = $derived.by(() => {
    const keySet = new Set<string>();
    for (let i = 0; i < drafts.length; i++) {
      const key = drafts[i][0];
      if (key == null) continue;
      if (keySet.has(key)) return i;
      keySet.add(key);
    }
  });

  $effect(() => {
    if (duplicate_key_index != null) {
      error = `Duplicate key '${drafts[duplicate_key_index][0]}'`;
      return;
    }

    // set the shown error to the first one
    error = errors.find((err) => err != null);
  });

  $effect(() => {
    userValue = Object.fromEntries(
      drafts.filter((draft) => draft[0] != null && draft[1] != null) as [
        string,
        JsonValue,
      ][],
    );
  });
</script>

<ul class="input-map">
  <li class="input-map-row input-map-header">
    <div class="input-map-corner"></div>
    <div class="key-label">Key</div>

    {#if "struct" in schema["value-type"]}
      <ul class="input-map-header-struct-fields">
        {#each Object.keys(schema["value-type"].struct.fields) as field (field)}
          <li class="input-map-header-struct-field">
            {field}
          </li>
        {/each}
      </ul>
    {/if}
  </li>

  {#each drafts as _, i}
    <li class="input-map-row">
      <div class="input-map-item-remove">
        <Button
          theme="secondary"
          onclick={() => {
            drafts.splice(i, 1);
            errors.splice(i, 1);
          }}
        >
          -
        </Button>
      </div>

      <!-- key -->
      <div class="key">
        <InputText
          schema={{
            "min-length": 1,
            "max-length": Infinity,
            default: null,
          }}
          bind:userValue={drafts[i][0]}
          bind:error={() =>
            duplicate_key_index === i
              ? `Duplicate key ${drafts[i][0]}`
              : errors[i],
          (err) => (errors[i] = err)}
        />
      </div>

      <div class="value">
        <InputField
          schema={schema["value-type"]}
          bind:userValue={drafts[i][1]}
          bind:error={errors[i]}
          showStructLabels={false}
        />
      </div>
    </li>
  {/each}
  <li class="input-map-add">
    <Button
      theme="secondary"
      stretch
      onclick={() => {
        drafts.push([undefined, undefined]);
        errors.push(undefined);
      }}
    >
      +
    </Button>
  </li>
</ul>

<style lang="scss">
  .input-map {
    list-style-type: none;
  }

  .input-map-row {
    display: grid;
    grid-template-columns: 2rem 5rem auto;
  }

  .input-map-item-remove {
    display: grid;
  }

  .input-map-add {
    min-width: 3rem;
  }

  .key-label,
  .input-map-header-struct-field {
    background: var(--color-surface-container);
    border-bottom: 2px solid var(--color-tertiary);
  }

  .input-map-header-struct-fields {
    list-style-type: none;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(0, 1fr));
  }

  .input-map-header {
    text-align: center;
    font-size: var(--fs-small);
    font-weight: bold;
  }
</style>
