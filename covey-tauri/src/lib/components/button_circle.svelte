<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";

  import Button, { type ButtonTheme } from "./button.svelte";

  type Props = HTMLButtonAttributes & {
    theme: ButtonTheme;
    children?: Snippet;
    button?: HTMLButtonElement;
    size: string;
  };

  let {
    theme,
    children,
    button = $bindable(),
    size,
    ...rest
  }: Props = $props();
</script>

<!-- @component
Button with a circular shape.

The button's size is not automatically set, you must specify a size.
 -->

<!-- the same element is required twice to force the button's size -->
<div class="button-circle-padding" style:--button-circle-size={size}>
  <Button {theme} rounding="full" bind:button {...rest}>
    <div class="button-circle-padding">
      {@render children?.()}
    </div>
  </Button>
</div>

<style lang="scss">
  .button-circle-padding {
    width: var(--button-circle-size);
    height: var(--button-circle-size);
    display: grid;
    place-content: center;
  }
</style>
