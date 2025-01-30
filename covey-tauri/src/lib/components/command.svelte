<script lang="ts">
  import type { Command, Hotkey } from "$lib/bindings";

  import InputHotkey from "./input_hotkey.svelte";

  let {
    command,
    userHotkey = $bindable(),
  }: {
    command: Command;
    userHotkey?: Hotkey;
  } = $props();
</script>

<div class="command">
  <p class="command-title">{command.title}</p>
  {#if command.description != null}
    <p class="command-description">{command.description}</p>
  {/if}

  <InputHotkey
    bind:userHotkey
    default={command["default-hotkey"] ?? undefined}
  />
</div>

<style lang="scss">
  .command {
    display: grid;
    gap: 0.5rem;
  }
</style>
