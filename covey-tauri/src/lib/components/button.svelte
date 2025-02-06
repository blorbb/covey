<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";

  type Props = HTMLButtonAttributes & {
    theme: "primary" | "secondary" | "tertiary" | "none";
    stretch?: boolean;
    pill?: boolean;
    children?: Snippet;
    button?: HTMLButtonElement;
    minWidth?: string;
  };

  let {
    theme,
    stretch = false,
    pill = false,
    children,
    button = $bindable(),
    ...rest
  }: Props = $props();
</script>

<button
  bind:this={button}
  class={["button", theme, { stretch, pill }]}
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

      &:hover {
        background-color: var(--color-surface-container-high);
      }
    }
  }
</style>
