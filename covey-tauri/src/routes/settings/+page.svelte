<script lang="ts">
  import Command from "$lib/components/command.svelte";

  import type { LayoutData } from "./$types";
  const { data }: { data: LayoutData } = $props();
  const appSettings = $derived(data.settings.globalConfig.app);
</script>

<Command
  command={{
    id: "reload-command",
    title: "Reload hotkey",
    description: "Hotkey to re-initialise the current plugin.",
    "default-hotkey": {
      ctrl: true,
      alt: false,
      shift: false,
      meta: false,
      key: "r",
    },
  }}
  bind:userHotkey={appSettings["reload-hotkey"]}
/>
