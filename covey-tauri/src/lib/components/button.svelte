<script module lang="ts">
  export type ButtonTheme =
    | "primary"
    | "secondary"
    | "tertiary"
    | "accent"
    | "error"
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
  class={["button", { stretch }]}
  data-theme={theme}
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

    &[data-theme="secondary"] {
      background-color: var(--color-secondary-container);
      color: var(--color-on-secondary-container);

      transition: var(--time-transition) filter;
      &:hover {
        filter: brightness(1.2);
      }
    }

    &[data-theme="tertiary"] {
      background-color: var(--color-surface-container);
      color: var(--color-on-surface-container);
      transition-property: background-color, color;
      transition-duration: var(--time-transition);

      &:hover {
        background-color: var(--color-surface-container-high);
        color: var(--color-on-surface-container);
      }
    }

    &[data-theme="accent"] {
      background-color: var(--color-tertiary);
      color: var(--color-on-tertiary);
      transition-property: background-color, color;
      transition-duration: var(--time-transition);

      &:hover {
        background-color: var(--color-tertiary-container);
        color: var(--color-on-tertiary-container);
      }
    }

    &[data-theme="error"] {
      background-color: var(--color-error);
      color: var(--color-on-error);
      transition-property: background-color, color;
      transition-duration: var(--time-transition);

      &:hover {
        background-color: var(--color-error-container);
        color: var(--color-on-error-container);
      }
    }
  }
</style>
