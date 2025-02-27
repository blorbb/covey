<script lang="ts">
  import type { Hotkey, Id } from "$lib/bindings";
  import * as keys from "$lib/keys";
  import type { CommandInfo } from "$lib/menu.svelte";
  import type { DeepReadonly } from "$lib/utils";

  import Button from "./button.svelte";
  import HotkeyKeys from "./hotkey_keys.svelte";

  let {
    commands,
    onActivate,
  }: {
    commands: DeepReadonly<CommandInfo[]>;
    onActivate: (commandId: Id) => void;
  } = $props();

  // two modifications on `commands`:
  // 1. use user hotkeys or default hotkeys
  // 2. if a previous command uses a hotkey, do not display it as an
  // available hotkey on subsequent commands
  const filteredCommandInfo = $derived.by(() => {
    const usedHotkeys: Hotkey[] = [];
    return commands.map((command) => {
      const allHotkeys =
        command.customHotkeys ?? command["default-hotkeys"] ?? [];

      const unusedHotkeys = allHotkeys.filter(
        (newHotkey) =>
          !usedHotkeys.some((oldHotkey) =>
            keys.hotkeysEqual(oldHotkey, newHotkey),
          ),
      );

      usedHotkeys.push(...unusedHotkeys);
      return {
        hotkeys: unusedHotkeys,
        command,
      };
    });
  });
</script>

<div class="menu-footer-commands">
  {#each filteredCommandInfo as info}
    <Button
      theme="tertiary"
      rounding="large"
      onclick={() => onActivate(info.command.id)}
    >
      <div class="footer-command-button">
        <div class="footer-command-button-hotkeys">
          {#each info.hotkeys as hotkey, i (`${info.command.id} : ${i}`)}
            <!-- separate by slashes -->
            {#if i !== 0}/{/if}
            <HotkeyKeys theme="tertiary" {hotkey} />
          {/each}
        </div>

        <span>
          {info.command.title}
        </span>
      </div>
    </Button>
  {/each}
</div>

<style lang="scss">
  .menu-footer-commands {
    display: flex;
    gap: 0.5rem;
  }

  .footer-command-button {
    padding: 0.25rem 0.5rem;
    display: flex;
    gap: 0.5rem;
  }

  .footer-command-button-hotkeys {
    display: flex;
    gap: 0.25rem;
  }
</style>
