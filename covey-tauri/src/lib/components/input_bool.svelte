<script lang="ts">
  import type { SchemaBool } from "$lib/bindings";

  let {
    schema,
    userValue = $bindable(),
    error = $bindable(),
  }: { schema: SchemaBool; userValue?: boolean; error?: string } = $props();

  error = undefined;
</script>

<input
  type="checkbox"
  class="input-bool"
  aria-invalid={error != null}
  bind:checked={() => userValue ?? schema.default ?? false,
  (checked) => (userValue = checked)}
/>

<style lang="scss">
  // TODO: style as toggle button
  .input-bool {
    width: 3rem;
  }
</style>
