<script lang="ts">
  import type { Hotkey } from "$lib/bindings";
  import * as keys from "$lib/keys";

  import Button from "./button.svelte";
  import ButtonCircle from "./button_circle.svelte";
  import HotkeyKeys from "./hotkey_keys.svelte";

  let {
    userHotkey = $bindable(),
    default: defaultHotkey,
    onDelete,
  }: {
    userHotkey?: Hotkey;
    default?: Hotkey;
    /**
     * Function to call when this hotkey is deleted.
     *
     * If this is not defined, there will be no delete button.
     */
    onDelete?: () => void;
  } = $props();

  let button = $state<HTMLButtonElement>();
  let capturing = $state(false);

  const newEmptyDraft = (): keys.MaybeHotkey => ({
    key: undefined,
    ctrl: false,
    alt: false,
    shift: false,
    meta: false,
  });
  let draft = $state(newEmptyDraft());

  let displayedHotkey = $derived(
    capturing ? draft : (userHotkey ?? defaultHotkey),
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

    const keyName = keys.symbolToKeyCode(e.key);
    if (keyName !== undefined) {
      // commit the key when a non-modifier is pressed
      userHotkey = { ...draft, key: keyName };
      button?.blur();
    }
  };
</script>

<span class="input-hotkey" class:capturing>
  <Button
    bind:button
    theme="none"
    onkeydown={registerKey}
    onclick={() => (capturing = true)}
    onblur={() => {
      capturing = false;
      draft = newEmptyDraft();
    }}
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
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 0.5rem;
    align-items: center;

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
  }

  .input-hotkey.capturing {
    border-bottom: 2px solid var(--color-primary-fixed);
  }

  .placeholder {
    color: var(--color-on-surface-variant);
  }

  .hotkey-x {
    font-size: 0.5rem;
  }
</style>
