<script module>
  export type ButtonTheme =
    | "primary"
    | "secondary"
    | "tertiary"
    | "accent"
    | "none";
  export type ButtonRounding = "full" | "large" | "small" | "none";
</script>

<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";

  type Props = HTMLButtonAttributes & {
    theme: ButtonTheme;
    stretch?: boolean;
    rounding?: ButtonRounding;
    children?: Snippet;
    button?: HTMLButtonElement;
    minWidth?: string;
  };

  let {
    theme,
    stretch = false,
    rounding = "none",
    children,
    button = $bindable(),
    ...rest
  }: Props = $props();
</script>

<button
  bind:this={button}
  class={["button", theme, { stretch }]}
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

    &[data-rounding="full"] {
      border-radius: 999999rem;
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

    &.accent {
      background-color: var(--color-tertiary);
      color: var(--color-on-tertiary);
      transition: var(--time-transition) filter;

      &:hover {
        filter: brightness(1.15);
      }
    }
  }
</style>
