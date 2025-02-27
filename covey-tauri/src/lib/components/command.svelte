<script lang="ts">
  import type { Command, Hotkey } from "$lib/bindings";
  import type { DeepReadonly } from "$lib/utils";

  import ButtonCircle from "./button_circle.svelte";
  import InputHotkey from "./input_hotkey.svelte";

  let {
    command,
    userHotkeys = $bindable(),
  }: {
    command: DeepReadonly<Command>;
    userHotkeys?: Hotkey[];
  } = $props();

  userHotkeys =
    userHotkeys ?? command["default-hotkeys"]?.map((x) => ({ ...x })) ?? [];

  let isAddingNewHotkey = $state(false);
</script>

<div class="command">
  <p class="command-title">{command.title}</p>
  {#if command.description != null}
    <p class="command-description">{command.description}</p>
  {/if}

  <div class="hotkeys">
    {#each userHotkeys as _, i}
      <InputHotkey
        userHotkey={userHotkeys[i]}
        onCommitUserHotkey={(x) => (userHotkeys[i] = x)}
        onDelete={() => userHotkeys.splice(i)}
      />
    {/each}

    <!-- + button for a new hotkey -->
    {#if isAddingNewHotkey}
      <!-- currently in progress of adding one -->
      <InputHotkey
        onCommitUserHotkey={(x) => {
          userHotkeys.push(x);
          isAddingNewHotkey = false;
        }}
        onCancel={() => (isAddingNewHotkey = false)}
        capturing={true}
      />
    {:else}
      <ButtonCircle
        theme="accent"
        size="1rem"
        onclick={() => (isAddingNewHotkey = true)}
      >
        <iconify-icon class="hotkey-add" icon="ph:plus-bold"></iconify-icon>
      </ButtonCircle>
    {/if}
  </div>
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

  .hotkeys {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .hotkey-add {
    font-size: 0.5rem;
  }
</style>
