<script lang="ts">
  import type { SchemaInt } from "$lib/bindings";

  let {
    schema,
    userValue = $bindable(),
    error = $bindable(),
  }: { schema: SchemaInt; userValue?: number; error?: string } = $props();

  const setValue = (str: string) => {
    // don't use parseInt as there are a lot of edge cases
    const value = Number(str);

    // empty string becomes 0, don't allow this
    if (isNaN(value) || str === "") {
      error = "Input is not a number";
      return;
    }

    if (!Number.isInteger(value)) {
      error = "Input must be an integer";
      return;
    }

    if (value < schema.min) {
      error = "Input is too small (must be at least {schema.min})";
      return;
    }

    if (value > schema.max) {
      error = "Input is too large (must be at most {schema.max})";
      return;
    }

    error = undefined;
    userValue = value;
  };
</script>

<input
  type="text"
  inputmode="numeric"
  class="input-int"
  aria-invalid={error != null}
  value={userValue?.toString() ?? schema.default?.toString() ?? ""}
  onchange={(ev) => setValue(ev.currentTarget.value)}
/>

<style lang="scss">
  .input-int {
    border-bottom: 2px solid var(--color-outline);
    background: var(--color-surface-container);
    padding: 0.25rem 0.5rem;
  }
</style>
