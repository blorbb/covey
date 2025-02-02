<script lang="ts">
  import type { SchemaText } from "$lib/bindings";

  let {
    schema,
    userValue = $bindable(),
    error = $bindable(),
  }: { schema: SchemaText; userValue?: string; error?: string } = $props();

  const setValue = (value: string) => {
    if (value.length < schema["min-length"]) {
      error = `Input is too short (must be at least ${schema["min-length"]} characters)`;
      return;
    }

    if (value.length > schema["max-length"]) {
      error = `Input is too long (must be at most ${schema["max-length"]} characters)`;
      return;
    }

    error = undefined;
    userValue = value;
  };
</script>

<input
  type="text"
  class="input-text"
  aria-invalid={error != null}
  value={userValue ?? schema.default ?? ""}
  onchange={(ev) => setValue(ev.currentTarget.value)}
/>

<style lang="scss">
  .input-text {
    border-bottom: 2px solid var(--color-outline);
    background: var(--color-surface-container);
    padding: 0.25rem 0.5rem;
  }
</style>
