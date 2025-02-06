<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";

  type Props = HTMLButtonAttributes & {
    theme: "primary" | "secondary" | "tertiary" | "none";
    stretch?: boolean;
    pill?: boolean;
    rounding?: "large" | "small" | "none";
    children?: Snippet;
    button?: HTMLButtonElement;
    minWidth?: string;
  };

  let {
    theme,
    stretch = false,
    pill = false,
    rounding = "none",
    children,
    button = $bindable(),
    ...rest
  }: Props = $props();
</script>

<button
  bind:this={button}
  class={["button", theme, { stretch, pill }]}
  data-rounding={rounding}
  {...rest}
>
  {@render children?.()}
</button>

<style lang="scss">
  .button {
    display: inline-grid;
    place-content: center;

    &.stretch {
      width: 100%;
    }

    &[data-rounding="small"] {
      border-radius: 0.25rem;
    }

    &[data-rounding="large"] {
      border-radius: 1rem;
    }

    &.pill {
      border-radius: 999rem;
      padding-inline: 0.5rem;
    }

    &.secondary {
      background-color: var(--color-secondary-container);
      color: var(--color-on-secondary-container);

      transition: var(--time-transition) filter;
      &:hover {
        filter: brightness(1.2);
      }
    }

    &.tertiary {
      transition-property: background-color, color;
      transition-duration: var(--time-transition);
      background-color: var(--color-surface-container);

      &:hover {
        background-color: var(--color-surface-container-high);
        color: var(--color-on-surface-container);
      }
    }
  }
</style>
