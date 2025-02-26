<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";

  import Button, { type ButtonTheme } from "./button.svelte";

  type Props = HTMLButtonAttributes & {
    theme: ButtonTheme;
    stretch?: boolean;
    children?: Snippet;
    button?: HTMLButtonElement;
    minWidth?: string;
  };

  let {
    theme,
    stretch = false,
    children,
    button = $bindable(),
    ...rest
  }: Props = $props();
</script>

<!-- @component
Button with full rounding and automatic inline padding.
 -->

<Button {theme} {stretch} rounding="full" bind:button {...rest}>
  <span class="button-pill-padding">
    {@render children?.()}
  </span>
</Button>

<style lang="scss">
  .button-pill-padding {
    padding-inline: 0.5em;
  }
</style>
