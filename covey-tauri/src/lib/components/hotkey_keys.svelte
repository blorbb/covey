<script lang="ts">
  import * as keys from "$lib/keys";
  import type { DeepReadonly } from "$lib/utils";

  let {
    hotkey,
    theme,
  }: {
    hotkey: DeepReadonly<keys.MaybeHotkey>;
    theme?: "secondary" | "tertiary";
  } = $props();
</script>

<div class={["hotkey", theme]}>
  {#if hotkey.ctrl}
    <kbd class="ctrl">Ctrl</kbd>
  {/if}
  {#if hotkey.alt}
    <kbd class="alt">Alt</kbd>
  {/if}
  {#if hotkey.shift}
    <kbd class="shift">Shift</kbd>
  {/if}
  {#if hotkey.meta}
    <kbd class="meta">Meta</kbd>
  {/if}
  {#if hotkey?.key}
    <kbd class="key">{keys.nameToSymbol(hotkey.key)}</kbd>
  {/if}
</div>

<style lang="scss">
  .hotkey {
    display: flex;
    gap: 0.5rem;

    kbd {
      font-family: var(--ff-mono);
      font-size: var(--fs-small);
      line-height: 1;
      padding: 0.25rem 0.5rem;
      border-radius: 0.5rem;
      display: inline-block;
    }

    &.secondary kbd {
      background: var(--color-secondary-container);
      color: var(--color-on-secondary-container);
      border-bottom: 2px solid var(--color-shadow);
    }

    &.tertiary kbd {
      background-color: var(--color-surface-container-highest);
      box-shadow: var(--shadow-small);
    }
  }
</style>
