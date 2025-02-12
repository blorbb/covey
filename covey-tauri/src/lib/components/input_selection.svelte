<script lang="ts">
  import type { SchemaSelection } from "$lib/bindings";
  import type { DeepReadonly } from "$lib/utils";

  let {
    schema,
    userValue = $bindable(),
    error = $bindable(),
  }: {
    schema: DeepReadonly<SchemaSelection>;
    userValue?: string;
    error?: string;
  } = $props();

  const setValue = (value: string) => {
    if (schema["allowed-values"].includes(value)) {
      error = undefined;
      userValue = value;
    } else {
      error =
        "Selection must be one of: " + schema["allowed-values"].join(", ");
    }
  };
</script>

<select
  class="input-selection"
  aria-invalid={error != null}
  value={userValue ?? schema.default ?? ""}
  onchange={(ev) => setValue(ev.currentTarget.value)}
>
  {#each schema["allowed-values"] as option}
    <option value={option}>{option}</option>
  {/each}
</select>

<style lang="scss">
  .input-selection {
    appearance: none;
    border-bottom: 2px solid var(--color-outline);
    background-color: var(--color-surface-container);
    padding: 0.25rem 0.5rem;

    // stolen from obsidian selection boxes
    // TODO: don't steal this lol
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='%23FFF' opacity='0.6' stroke-width='2' stroke-linecap='round' stroke-linejoin='round' %3E%3Cpath d='m7 15 5 5 5-5'/%3E%3Cpath d='m7 9 5-5 5 5'/%3E%3C/svg%3E");
    background-repeat: no-repeat, repeat;
    background-position:
      right 0.5em top 50%,
      0 0;
    background-size:
      1em auto,
      100%;
    background-blend-mode: hard-light;
  }
</style>
