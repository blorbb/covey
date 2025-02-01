<script lang="ts">
  import type { SchemaText } from "$lib/bindings";
  import { unreachable } from "$lib/utils";

  let {
    schema,
    userValue = $bindable(),
  }: { schema: SchemaText; userValue?: string } = $props();

  // unvalidated value
  let draft = $state(userValue ?? schema.default ?? "");
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

    error = undefined;

    userValue = draft;
  };
</script>

<div class="input-text">
  <input
    type="text"
    class="input-text-input"
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
  {:else if error === undefined}{:else}
    {unreachable(error)}
  {/if}
</div>

<style lang="scss">
  .input-text-input {
    border: 2px solid var(--color-outline);
    background: var(--color-surface-container);
    padding: 0.25rem 0.5rem;
  }

  .error-message {
    color: var(--color-error);
  }
</style>
