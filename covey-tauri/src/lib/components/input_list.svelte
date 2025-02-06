<script lang="ts">
  import type { JsonValue, SchemaList } from "$lib/bindings";
  import type { DeepReadonly } from "$lib/utils";

  import Button from "./button.svelte";
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
      <div class="input-list-item-remove">
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

      <InputField
        schema={schema["item-type"]}
        bind:userValue={drafts[i]}
        bind:error={errors[i]}
      />
    </li>
  {/each}
  <li class="input-list-add">
    <Button
      theme="secondary"
      stretch
      onclick={() => {
        drafts.push(undefined);
        errors.push(undefined);
      }}
    >
      +
    </Button>
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
    display: grid;
  }

  .input-list-add {
    min-width: 3rem;
  }
</style>
