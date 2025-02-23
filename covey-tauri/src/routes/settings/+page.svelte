<script lang="ts">
  import Command from "$lib/components/command.svelte";
  import Config from "$lib/components/config.svelte";

  import type { LayoutData } from "./$types";
  const { data }: { data: LayoutData } = $props();
  const appSettings = $derived(data.settings.globalConfig.app);
</script>

<div class="app-settings">
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

  <Config
    schema={{
      id: "icon-themes",
      title: "Named icon preferences",
      description:
        "Icons to try and use when given a named icon.\nIcons will be chosen from top to bottom. They can be from the system, or a prefix from iconify-icon.",
      type: {
        list: {
          "min-items": 0,
          unique: false,
          "item-type": {
            struct: {
              fields: {
                kind: {
                  selection: {
                    "allowed-values": ["system", "iconify-icon"],
                    default: null,
                  },
                },
                name: {
                  text: {
                    "min-length": 0,
                    "max-length": Number.MAX_SAFE_INTEGER,
                    default: null,
                  },
                },
              },
            },
          },
        },
      },
    }}
    bind:userValue={appSettings["icon-themes"]}
  />
</div>

<style lang="scss">
  .app-settings {
    display: grid;
    gap: 2rem;
  }
</style>
