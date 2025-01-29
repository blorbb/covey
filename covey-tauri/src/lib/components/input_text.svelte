<script lang="ts">
  import type { SchemaText } from "$lib/bindings";

  let { schema, value = $bindable() }: { schema: SchemaText; value?: string } =
    $props();

  // unvalidated value
  let draft = $state(value ?? schema.default ?? "");
  let error: undefined | "short" | "long" = $state();

  const commitDraft = () => {
    if (draft.length < schema["min-length"]) {
      error = "short";
      return;
    }

    if (draft.length > schema["max-length"]) {
      error = "long";
      return;
    }

    error = undefined

    value = draft;
  };
</script>

<div class="input-str">
  <input
    type="text"
    class="input-str-input"
    aria-invalid={error != null}
    bind:value={draft}
    onchange={commitDraft}
  />
  {#if error === "short"}
    <div class="error-message">
      Input is too short (must be at least {schema["min-length"]} characters)
    </div>
  {:else if error === "long"}
    <div class="error-message">
      Input is too long (must be at most {schema["max-length"]} characters)
    </div>
  {/if}
</div>

<style lang="scss">
  .input-str-input {
    border: 2px solid var(--color-outline);
    background: var(--color-surface-container);
  }
</style>