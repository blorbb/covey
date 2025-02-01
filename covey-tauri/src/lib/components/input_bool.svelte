<script lang="ts">
  import type { SchemaBool } from "$lib/bindings";

  let {
    schema,
    userValue = $bindable(),
  }: { schema: SchemaBool; userValue?: boolean } = $props();
</script>

<div class="input-bool">
  <input
    type="checkbox"
    class="input-bool-input"
    bind:checked={() => userValue ?? schema.default ?? false,
    (checked) => (userValue = checked)}
  />
</div>

<style lang="scss">
  // TODO: style as a toggle button
</style>
