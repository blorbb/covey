<script lang="ts">
  import type { SchemaText } from "$lib/bindings";

  let {
    schema,
    userValue = $bindable(),
    error = $bindable(),
  }: { schema: SchemaText; userValue?: string; error?: string } = $props();

  // unvalidated value
  let draft = $state(userValue ?? schema.default ?? "");

  const commitDraft = () => {
    if (draft.length < schema["min-length"]) {
      error = `Input is too short (must be at least ${schema["min-length"]} characters)`;
      return;
    }

    if (draft.length > schema["max-length"]) {
      error = `Input is too long (must be at most ${schema["max-length"]} characters)`;
      return;
    }

    error = undefined;
    userValue = draft;
  };
</script>

<input
  type="text"
  class="input-text"
  aria-invalid={error != null}
  bind:value={draft}
  onchange={commitDraft}
/>

<style lang="scss">
  .input-text {
    border: 2px solid var(--color-outline);
    background: var(--color-surface-container);
    padding: 0.25rem 0.5rem;
    min-width: 0;
    width: 100%;
  }
</style>
