<script lang="ts">
  import type { SchemaInt } from "$lib/bindings";
  import { unreachable } from "$lib/utils";

  let {
    schema,
    userValue = $bindable(),
  }: { schema: SchemaInt; userValue?: number } = $props();

  // unvalidated value
  let draft = $state(userValue?.toString() ?? schema.default?.toString() ?? "");
  let error: undefined | "small" | "large" | "float" | "nan" = $state();

  const commitDraft = () => {
    // don't use parseInt as there are a lot of edge cases
    const value = Number(draft);
    if (isNaN(value)) {
      error = "nan";
      return;
    }

    if (!Number.isInteger(value)) {
      error = "float";
      return;
    }

    if (value < schema.min) {
      error = "small";
      return;
    }

    if (value > schema.max) {
      error = "large";
      return;
    }

    error = undefined;
    userValue = value;
  };
</script>

<div class="input-int">
  <input
    type="text"
    inputmode="numeric"
    class="input-int-input"
    aria-invalid={error != null}
    bind:value={draft}
    onchange={commitDraft}
  />
  {#if error === "small"}
    <div class="error-message">
      Input is too small (must be at least {schema.min})
    </div>
  {:else if error === "large"}
    <div class="error-message">
      Input is too large (must be at most {schema.max})
    </div>
  {:else if error === "float"}
    <div class="error-message">Input must be an integer</div>
  {:else if error === "nan"}
    <div class="error-message">Input is not a number</div>
  {:else if error === undefined}{:else}
    {unreachable(error)}
  {/if}
</div>

<style lang="scss">
  .input-int-input {
    border: 2px solid var(--color-outline);
    background: var(--color-surface-container);
  }

  .error-message {
    color: var(--color-error);
  }
</style>
