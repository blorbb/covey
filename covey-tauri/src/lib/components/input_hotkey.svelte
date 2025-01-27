<script lang="ts">
  import type { Hotkey, Key } from "$lib/bindings";
  import * as keys from "$lib/keys";

  let {
    currentHotkey,
    default: defaultHotkey,
    setHotkey,
  }: {
    currentHotkey?: Hotkey;
    default?: Hotkey;
    setHotkey: (hotkey: Hotkey) => void;
  } = $props();

  let button = $state<HTMLButtonElement>();
  let capturing = $state(false);

  const emptyDraft = () => ({
    key: undefined as Key | undefined,
    ctrl: false,
    alt: false,
    shift: false,
    meta: false,
  });
  let draft = $state(emptyDraft());

  let displayedHotkey = $derived(
    capturing ? draft : (currentHotkey ?? defaultHotkey),
  );

  let emptyHotkey = $derived(
    displayedHotkey === undefined ||
      (displayedHotkey.key === undefined &&
        !displayedHotkey.ctrl &&
        !displayedHotkey.alt &&
        !displayedHotkey.shift &&
        !displayedHotkey.meta),
  );

  const registerKey = (e: KeyboardEvent) => {
    if (!capturing) return;

    e.preventDefault();

    if (e.key === "Escape") {
      button?.blur();
    }

    draft.ctrl = e.ctrlKey;
    draft.alt = e.altKey;
    draft.shift = e.shiftKey;
    draft.meta = e.metaKey;

    const keyName = keys.symbolToName(e.key);
    if (keyName !== undefined) {
      // commit the key when a non-modifier is pressed
      setHotkey({ ...draft, key: keyName });
      button?.blur();
    }
  };
</script>

<button
  bind:this={button}
  class="input-hotkey"
  class:capturing
  class:empty={emptyHotkey}
  onkeydown={registerKey}
  onclick={() => (capturing = true)}
  onblur={() => {
    capturing = false;
    draft = emptyDraft();
  }}
>
  {#if displayedHotkey?.ctrl}
    <kbd class="ctrl">Ctrl</kbd>
  {/if}
  {#if displayedHotkey?.alt}
    <kbd class="alt">Alt</kbd>
  {/if}
  {#if displayedHotkey?.shift}
    <kbd class="shift">Shift</kbd>
  {/if}
  {#if displayedHotkey?.meta}
    <kbd class="meta">Meta</kbd>
  {/if}
  {#if displayedHotkey?.key}
    <kbd class="key">{keys.nameToSymbol(displayedHotkey.key)}</kbd>
  {/if}
</button>

<style lang="scss">
  .input-hotkey {
    padding: 0.25rem;
    min-width: 1rem;
    width: max-content;
    // calculate from padding/font size/line height of the keyboard
    // need to make it stay the same when empty
    height: 2rem;

    border-bottom: 2px solid var(--color-outline-variant);
    box-shadow: var(--shadow-small);

    user-select: none;
    cursor: pointer;

    display: flex;
    gap: 0.5rem;

    &.empty::before {
      content: "Enter hotkey...";
      color: var(--color-on-surface-variant);
    }
  }

  .input-hotkey.capturing {
    border-bottom: 2px solid var(--color-primary-fixed);

    kbd {
      background: var(--color-secondary);
      color: var(--color-on-secondary);
    }
  }

  kbd {
    background: var(--color-secondary-container);
    color: var(--color-on-secondary-container);
    border-bottom: 2px solid var(--color-shadow);

    font-family: var(--ff-mono);
    font-size: var(--fs-small);
    line-height: 1;
    padding: 0.25rem 0.5rem;
    border-radius: 0.5rem;
    display: inline-block;
  }
</style>
