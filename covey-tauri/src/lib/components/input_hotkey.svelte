<script lang="ts">
  import type { Hotkey } from "$lib/bindings";
  import * as keys from "$lib/keys";

  import Button from "./button.svelte";
  import HotkeyKeys from "./hotkey_keys.svelte";

  let {
    userHotkey = $bindable(),
    default: defaultHotkey,
  }: {
    userHotkey?: Hotkey;
    default?: Hotkey;
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
</span>

<style lang="scss">
  .input-hotkey {
    display: inline-block;
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
</style>
