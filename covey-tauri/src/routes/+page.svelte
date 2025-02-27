<script lang="ts">
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onDestroy } from "svelte";

  import type { Id, ListStyle } from "$lib/bindings";
  import Button from "$lib/components/button.svelte";
  import ButtonCircle from "$lib/components/button_circle.svelte";
  import MenuFooterCommands from "$lib/components/menu_footer_commands.svelte";
  import ScrollShadow from "$lib/components/scroll_shadow.svelte";

  import type { PageData } from "./$types";

  const { data }: { data: PageData } = $props();
  const menu = data.menu;
  const iconCache = data.iconCache;

  // global keyboard events
  const windowKeyDown = (ev: KeyboardEvent) => {
    switch (ev.key) {
      case "ArrowDown":
        menu.selection = Math.min(menu.items.length - 1, menu.selection + 1);
        ev.preventDefault();
        break;
      case "ArrowUp":
        menu.selection = Math.max(0, menu.selection - 1);
        ev.preventDefault();
        break;
      case "Escape":
        void getCurrentWindow().hide();
        ev.preventDefault();
        break;
      default: {
        const didActivate = menu.activateByEvent(ev);
        if (didActivate) ev.preventDefault();
      }
    }
  };

  // query on input change
  $effect(() => {
    // tracks menu.inputText
    menu.query();
  });

  // retain focus on the input element
  let mainInput = $state<HTMLInputElement>();
  let activeElement = $state<Element>();

  $effect(() => {
    if (activeElement !== mainInput) {
      mainInput?.focus();
    }
  });

  // react to selection updates
  $effect(() => {
    mainInput?.setSelectionRange(menu.textSelection[0], menu.textSelection[1]);
  });

  // select full input when focussed
  let unlisten: UnlistenFn | undefined;
  void listen("tauri://focus", () => {
    mainInput?.setSelectionRange(0, mainInput.value.length);
  }).then((f) => (unlisten = f));

  onDestroy(() => unlisten?.());

  // hide window when clicking outside the menu
  let menuWrapper = $state<HTMLElement>();
  const onPositionerPointerDown = (ev: PointerEvent) => {
    if (!(ev.target instanceof Node)) {
      return;
    }

    // hide window if clicked outside menu wrapper
    if (!menuWrapper?.contains(ev.target)) {
      void getCurrentWindow().hide();
    }
  };

  // menu.style with `undefined` configured to some default.
  // TODO: take the default from some setting
  let configuredListStyle: ListStyle = $derived(menu.style ?? { kind: "rows" });
  let listKind: "rows" | "grid" = $derived.by(() => {
    switch (configuredListStyle.kind) {
      case "rows":
      case "grid":
        return configuredListStyle.kind;
      case "gridWithColumns":
        return "grid";
    }
  });
  let listColumns = $derived.by(() => {
    switch (configuredListStyle.kind) {
      case "rows":
        return 1;
      case "grid":
        return 4; // TODO: make this configurable
      case "gridWithColumns":
        return configuredListStyle.columns;
    }
  });

  const navSettings = async (_plugin?: Id) => {
    await menu.showSettingsWindow();
    // TODO: navigate settings window to the plugin's settings
    // if a plugin is specified
  };
</script>

<svelte:document bind:activeElement />
<svelte:window onkeydown={windowKeyDown} />

<div class="positioner" onpointerdown={onPositionerPointerDown}>
  <div class="menu-wrapper" bind:this={menuWrapper}>
    <main class="menu">
      <div class="search-bar">
        <input
          class="search-input"
          type="text"
          bind:value={menu.inputText}
          bind:this={mainInput}
          placeholder="Search..."
        />
        <div class="settings-button">
          <ButtonCircle
            theme="tertiary"
            size="1lh"
            onclick={() => navSettings()}
          >
            <iconify-icon icon="ph:gear-bold"></iconify-icon>
          </ButtonCircle>
        </div>
      </div>
      <ScrollShadow>
        <div
          class="list"
          style:--list-columns={listColumns}
          data-list-style={listKind}
        >
          {#each menu.items as { id, description, title, icon }, i (id)}
            <label class="list-item">
              <input
                class="list-item-radio"
                type="radio"
                name="result-list"
                value={i}
                bind:group={menu.selection}
                onclick={(e) => {
                  // bind:group does not update selection fast enough
                  menu.selection = i;
                  void menu.activateByHotkey({
                    key: "enter",
                    ctrl: e.ctrlKey,
                    alt: e.altKey,
                    shift: e.shiftKey,
                    meta: e.metaKey,
                  });
                }}
              />
              <div class="icon">
                {#if icon?.kind === "text"}
                  <span class="icon-text">{icon.text}</span>
                {:else if icon?.kind === "file"}
                  {#await iconCache.open(icon.path) then src}
                    <img class="icon-img" {src} alt={`icon of ${title}`} />
                  {:catch err}
                    <div class="icon-error">
                      <!-- TODO: something here? -->
                      {err}
                    </div>
                  {/await}
                {/if}
              </div>
              <p class="title">{title}</p>
              <p class="description">{description}</p>
            </label>
          {/each}
        </div>
      </ScrollShadow>

      <div class="menu-footer">
        <MenuFooterCommands
          commands={menu.getAvailableCommands()}
          onActivate={(id) => menu.activateById(id)}
        />

        <div class="menu-footer-plugin-info">
          {#if menu.activePlugin != null}
            {@const manifest = menu.manifestOf(menu.activePlugin)}
            {#if manifest != null}
              <Button
                theme="tertiary"
                rounding="large"
                onclick={() => navSettings()}
              >
                <div class="footer-plugin-button">
                  {manifest.name}
                </div>
              </Button>
            {/if}
          {/if}
        </div>
      </div>
    </main>
  </div>
</div>

<style lang="scss">
  .menu-wrapper {
    border-radius: var(--brad-standard);
    border: 0.25rem solid var(--color-outline);
    overflow: hidden;
    position: relative;
    color: var(--color-on-surface);

    // blurred background image
    // window that blurs against the desktop background isn't
    // well supported, so background image looks nicer.
    &::before {
      content: "";
      position: absolute;
      inset: 0;
      z-index: -1;

      // credit: https://unsplash.com/photos/worms-eye-view-of-mountain-during-daytime-ii5JY_46xH0
      background-image: url("/background.jpg");
      background-size: cover;
      filter: blur(0.5vw);
    }
  }

  .menu {
    background: var(--color-surface);
    opacity: 0.93;

    width: 800px;
    max-height: 600px;
    @include grid-container();
    grid-template-rows: auto 1fr auto;
  }

  .search-bar {
    font-size: var(--fs-large);
    padding: 2rem;
    gap: 1rem;
    display: flex;
    flex-direction: row;
  }

  .search-input {
    flex-grow: 1;
    color: var(--color-on-surface);
    outline: none;

    &::placeholder {
      color: var(--color-on-surface-variant);
    }
  }

  .settings-button {
    display: grid;
  }

  .list {
    @include grid-container();
    gap: 1rem;
    padding: 1rem;
    grid-template-columns: repeat(var(--list-columns, 1), 1fr);

    &:empty {
      display: none;
    }
  }

  .list-item {
    // don't make these actual gap properties as each
    // area may not be defined. use margins instead.
    --_row-gap: 0.5rem;
    --_icon-gap: 1rem;

    padding: 1rem;
    border-radius: var(--brad-standard);

    @include grid-container();
    grid-template-areas: "icon title" "icon description";
    grid-template-columns: auto 1fr;

    // grid style
    .list[data-list-style="grid"] & {
      grid-template-areas: "icon" "title" "description";
      grid-template-columns: unset;
      justify-items: center;
      // align to top so that if some items in a row have
      // titles/descriptions that wrap across multiple lines,
      // shorter items generally align better vertically.
      align-content: start;

      .icon {
        margin-right: 0;
        margin-bottom: var(--_icon-gap);
      }

      // item with no icon looks better if centered
      &:has(.icon:empty, .icon-text:empty) {
        align-content: center;
      }
    }

    .icon {
      // make it take up the same size as a list item
      // with one row for the title + description
      --_icon-size: calc(
        (var(--fs-standard) + var(--fs-small)) * var(--line-height) +
          var(--_row-gap)
      );

      grid-area: icon;
      display: grid;
      place-content: center;
      width: var(--_icon-size);
      // needs to be a margin here instead of column-gap
      // so that no icon doesn't add a column
      margin-right: var(--_icon-gap);

      .icon-img {
        width: var(--_icon-size);
      }

      .icon-text {
        font-size: calc(var(--_icon-size) / var(--line-height));
      }

      &:empty,
      &:has(.icon-text:empty) {
        display: none;
      }
    }

    .title {
      grid-area: title;
      font-weight: bold;
      white-space: pre-line;
    }

    .description {
      grid-area: description;
      white-space: pre-line;
      font-size: var(--fs-small);
      color: var(--color-on-surface-variant);
      margin-top: var(--_row-gap);

      &:empty {
        display: none;
      }
    }

    &:hover {
      background: var(--color-surface-container-high);
    }

    &:has(.list-item-radio:checked) {
      background: var(--color-surface-container-highest);
    }
  }

  .list-item-radio {
    display: none;
  }

  .menu-footer {
    background: var(--color-surface);
    display: flex;
    padding: 0.5rem 1rem;
    justify-content: space-between;
  }

  .footer-plugin-button {
    padding: 0.25rem 0.5rem;
  }

  .positioner {
    display: grid;
    place-items: center;
    width: 100vw;
    height: 100vh;
  }

  // text in the menu should not be selectable
  .menu-wrapper :global(*) {
    user-select: none;
  }
</style>
