<script lang="ts">
  import type { Snippet } from "svelte";

  import { page } from "$app/state";
  import Divider from "$lib/components/divider.svelte";
  import DndList from "$lib/components/dnd_list.svelte";

  import type { LayoutData } from "./$types";

  const { data, children }: { data: LayoutData; children: Snippet } = $props();
  const settings = data.settings;
</script>

<main class="settings-layout">
  <nav class="settings-nav">
    <div class="global-settings">
      <a
        href="/settings"
        aria-current={page.url.pathname === "/settings" && "page"}
      >
        Global settings
      </a>
    </div>
    <Divider margin="0.5rem" />
    <h3>Plugins</h3>
    <div class="plugin-list">
      <DndList
        bind:items={settings.plugins}
        key={(item) => item.name}
      >
        {#snippet item({ name })}
          {@const url = `/settings/${encodeURIComponent(name)}`}

          <a
            class="plugin-list-item"
            href={url}
            aria-current={page.url.pathname === url && "page"}
          >
            <iconify-icon icon="ph:dots-six-vertical-bold"></iconify-icon>
            {name}
          </a>
        {/snippet}
      </DndList>
    </div>
  </nav>
  <div class="settings-content">
    {@render children()}
  </div>
</main>

<style lang="scss">
  .settings-layout {
    background: var(--color-surface);
    color: var(--color-on-surface);
    width: 100vw;
    height: 100vh;

    display: flex;
    gap: 2rem;
    padding: 2rem;

    // bigger line height for blocks of text
    line-height: 1.5;
  }

  .settings-nav {
    flex: 0 0 15rem;
    display: grid;
    align-content: start;

    h3 {
      padding: 0.5rem;
    }

    .plugin-list {
      overflow: auto;
    }

    a {
      display: block;
      padding: 0.5rem;
      padding-inline-end: 2rem;
      border-radius: 0.5rem;
      text-decoration: none;

      display: flex;
      align-items: center;
      gap: 0.5rem;

      &:hover {
        background: var(--color-surface-bright);
      }

      &[aria-current="page"] {
        background-color: var(--color-primary-container);
        color: var(--color-on-primary-container);
      }
    }
  }

  .settings-content {
    flex: 1 1 auto;
  }
</style>
