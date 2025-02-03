<script lang="ts">
  import type { JsonValue, SchemaMap } from "$lib/bindings";
  import type { DeepReadonly } from "$lib/utils";

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
  {#each drafts as _, i}
    <li class="input-map-item">
      <button
        class="input-map-item-remove"
        onclick={() => {
          drafts.splice(i, 1);
          errors.splice(i, 1);
        }}
      >
        -
      </button>

      <!-- key -->
      <div class="key">
        <InputText
          schema={{
            "min-length": 1,
            "max-length": Infinity,
            default: null,
          }}
          bind:userValue={drafts[i][0]}
          error={duplicate_key_index === i
            ? `Duplicate key ${drafts[i][0]}`
            : undefined}
        />
      </div>

      <div class="value">
        <InputField
          schema={schema["value-type"]}
          bind:userValue={drafts[i][1]}
          bind:error={errors[i]}
        />
      </div>
    </li>
  {/each}
  <li class="input-map-add">
    <button
      class="input-map-add-button"
      onclick={() => {
        drafts.push([undefined, undefined]);
        errors.push(undefined);
      }}
    >
      +
    </button>
  </li>
</ul>

<style lang="scss">
  .input-map {
    list-style-type: none;
  }

  .input-map-item {
    display: grid;
    grid-template-columns: 2rem 5rem auto;
  }

  .input-map-item-remove {
    background-color: var(--color-secondary-container);
    color: var(--color-on-secondary-container);
    transition: var(--time-transition) filter;
    &:hover {
      filter: brightness(1.2);
    }
  }

  .input-map-add-button {
    background-color: var(--color-secondary-container);
    color: var(--color-on-secondary-container);
    min-width: 3rem;

    transition: var(--time-transition) filter;
    &:hover {
      filter: brightness(1.2);
    }
  }
</style>
