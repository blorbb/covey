<script lang="ts">
  import type { Command, Hotkey } from "$lib/bindings";
  import type { DeepReadonly } from "$lib/utils";

  import InputHotkey from "./input_hotkey.svelte";

  let {
    command,
    userHotkeys = $bindable(),
  }: {
    command: DeepReadonly<Command>;
    userHotkeys?: Hotkey[];
  } = $props();

  let drafts = $state(
    userHotkeys ?? command["default-hotkeys"]?.map((x) => ({ ...x })) ?? [],
  );
  $effect(() => {
    userHotkeys = drafts;
  });
</script>

<div class="command">
  <p class="command-title">{command.title}</p>
  {#if command.description != null}
    <p class="command-description">{command.description}</p>
  {/if}

  {#each drafts as _, i}
    <InputHotkey bind:userHotkey={drafts[i]} />
  {/each}
  <!-- TODO: button to add extra hotkeys -->
</div>

<style lang="scss">
  .command {
    display: grid;
    gap: 0.5rem;
  }

  .command-title {
    font-weight: bold;
  }

  .command-description {
    color: var(--color-on-surface-variant);
    font-size: var(--fs-small);
  }
</style>
