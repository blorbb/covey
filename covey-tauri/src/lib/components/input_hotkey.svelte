<script lang="ts">
  import type { Hotkey } from "$lib/bindings";
  import * as keys from "$lib/keys";

  import Button from "./button.svelte";
  import ButtonCircle from "./button_circle.svelte";
  import HotkeyKeys from "./hotkey_keys.svelte";

  let {
    userHotkey,
    onCommitUserHotkey,
    onCancel,
    default: defaultHotkey,
    onDelete,
    capturing = $bindable(false),
  }: {
    userHotkey?: Hotkey;
    onCommitUserHotkey: (hotkey: Hotkey) => void;
    onCancel?: () => void;
    default?: Hotkey;
    /**
     * Function to call when this hotkey is deleted.
     *
     * If this is not defined, there will be no delete button.
     */
    onDelete?: () => void;
    /**
     * Whether this component is currently capturing a hotkey.
     */
    capturing?: boolean;
  } = $props();

  let button = $state<HTMLButtonElement>();

  let draft = $state(keys.newEmpty());

  let displayedHotkey = $derived(
    capturing ? draft : (userHotkey ?? defaultHotkey),
  );

  const registerKey = (e: KeyboardEvent) => {
    if (!capturing) return;

    e.preventDefault();

    if (e.key === "Escape") {
      capturing = false;
    }

    draft.ctrl = e.ctrlKey;
    draft.alt = e.altKey;
    draft.shift = e.shiftKey;
    draft.meta = e.metaKey;

    const keyName = keys.symbolToKeyCode(e.key);
    if (keyName !== undefined) {
      const newHotkey = { ...draft, key: keyName };
      console.debug("committing key", $state.snapshot(newHotkey));
      // commit the key when a non-modifier is pressed
      onCommitUserHotkey(newHotkey);
      button?.blur();
    }
  };

  $effect(() => {
    if (capturing) {
      button?.focus();
    } else {
      button?.blur();
      draft = keys.newEmpty();
      onCancel?.();
    }
  });
</script>

<span class="input-hotkey" class:capturing>
  <Button
    bind:button
    theme="none"
    onkeydown={registerKey}
    onkeyup={registerKey}
    onclick={() => (capturing = true)}
    onblur={() => (capturing = false)}
  >
    {#if displayedHotkey === undefined || keys.isEmpty(displayedHotkey)}
      <span class="placeholder">Enter hotkey...</span>
    {:else}
      <HotkeyKeys theme="secondary" hotkey={displayedHotkey} />
    {/if}
  </Button>
  {#if onDelete}
    <ButtonCircle theme="accent" size="1rem" onclick={onDelete}>
      <iconify-icon class="hotkey-x" icon="ph:x-bold"></iconify-icon>
    </ButtonCircle>
  {/if}
</span>

<style lang="scss">
  .input-hotkey {
    display: flex;
    gap: 0.5rem;
    align-items: center;

    padding: 0.25rem;
    min-width: 1rem;
    width: max-content;
    // calculate from padding/font size/line height of the keyboard
    // need to make it stay the same when empty
    height: 2rem;

    // include both a top and bottom border to be symmetrical
    border-block: 2px solid transparent;
    border-bottom-color: var(--color-outline-variant);

    box-shadow: var(--shadow-small);

    user-select: none;
    cursor: pointer;
  }

  .input-hotkey.capturing {
    border-bottom-color: var(--color-outline-variant);
  }

  .placeholder {
    color: var(--color-on-surface-variant);
  }

  .hotkey-x {
    font-size: 0.5rem;
  }
</style>
