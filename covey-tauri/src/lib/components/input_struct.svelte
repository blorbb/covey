<script lang="ts">
  import type { JsonValue, SchemaStruct } from "$lib/bindings";
  import type { DeepReadonly } from "$lib/utils";

  import InputField from "./input_field.svelte";

  let {
    schema,
    userValue = $bindable(),
    error = $bindable(),
  }: {
    schema: DeepReadonly<SchemaStruct>;
    userValue?: Record<string, JsonValue>;
    error?: string;
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

<ul class="input-map">
  {#each drafts as _, i}
    <li class="input-map-item">
      <InputField
        schema={schema.fields[drafts[i][0]]}
        bind:userValue={drafts[i][1]}
        bind:error={errors[i]}
      />
    </li>
  {/each}
</ul>

<style lang="scss">
  .input-map {
    list-style-type: none;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(0, 1fr));
  }
</style>
