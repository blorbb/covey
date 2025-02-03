<script lang="ts">
  import type { JsonValue, SchemaList } from "$lib/bindings";
  import type { DeepReadonly } from "$lib/utils";

  import InputField from "./input_field.svelte";

  let {
    schema,
    userValue = $bindable(),
    error = $bindable(),
  }: {
    schema: DeepReadonly<SchemaList>;
    userValue?: JsonValue[];
    error?: string;
  } = $props();

  let drafts: (JsonValue | undefined)[] = $state(
    userValue == null ? [] : [...userValue],
  );

  // error for each of the values
  // must be kept the same length as `drafts`
  let errors = $state<(string | undefined)[]>(drafts.map(() => undefined));

  // set the shown error to the first one
  $effect(() => {
    error = errors.find((err) => err != null);
  });

  $effect(() => {
    userValue = drafts.filter((draft) => draft != null);
  });
</script>

<ul class="input-list">
  {#each drafts as _, i}
    <li class="input-list-item">
      <button
        class="input-list-item-remove"
        onclick={() => {
          drafts.splice(i, 1);
          errors.splice(i, 1);
        }}
      >
        -
      </button>
      <InputField
        schema={schema["item-type"]}
        bind:userValue={drafts[i]}
        bind:error={errors[i]}
      />
    </li>
  {/each}
  <li class="input-list-add">
    <button
      class="input-list-add-button"
      onclick={() => {
        drafts.push(undefined);
        errors.push(undefined);
      }}
    >
      +
    </button>
  </li>
</ul>

<style lang="scss">
  .input-list {
    list-style-type: none;
  }

  .input-list-item {
    display: grid;
    grid-template-columns: 2rem 1fr;
  }

  .input-list-item-remove {
    background-color: var(--color-secondary-container);
    color: var(--color-on-secondary-container);
    transition: var(--time-transition) filter;
    &:hover {
      filter: brightness(1.2);
    }
  }

  .input-list-add-button {
    background-color: var(--color-secondary-container);
    color: var(--color-on-secondary-container);
    min-width: 3rem;

    transition: var(--time-transition) filter;
    &:hover {
      filter: brightness(1.2);
    }
  }
</style>
