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
  .input-bool {
    --toggle-width: 3rem;
    --toggle-height: 1.5rem;
    --toggle-thumb-size: 1rem;

    /// Distance from the edge to the outside of the thumb
    @function toggle-thumb-inset() {
      @return calc((var(--toggle-height) - var(--toggle-thumb-size)) / 2);
    }

    width: var(--toggle-width);
    height: var(--toggle-height);

    display: block;
    appearance: none;
    position: relative;

    background-color: var(--color-surface-container-highest);
    border-radius: var(--toggle-height);

    transition: var(--time-transition) background-color;

    &::after {
      content: "";
      position: absolute;
      left: 0;
      top: toggle-thumb-inset();
      width: var(--toggle-thumb-size);
      height: var(--toggle-thumb-size);
      border-radius: var(--toggle-thumb-size);

      background-color: var(--color-outline);
      transform: translateX(toggle-thumb-inset());

      transition-duration: var(--time-transition);
      transition-property: background-color, transform;
    }

    &:checked {
      background-color: var(--color-secondary);

      &::after {
        background-color: var(--color-on-secondary);

        // really bad formatting for some reason
        // prettier-ignore
        transform: translateX(
          calc(var(--toggle-width) - var(--toggle-thumb-size) - toggle-thumb-inset())
        );
      }
    }
  }
</style>
