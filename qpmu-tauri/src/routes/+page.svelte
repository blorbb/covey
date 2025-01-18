<script lang="ts">
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import * as commands from "$lib/commands";
  import { onDestroy } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import type { PageData } from "./$types";
  import ScrollShadow from "$lib/components/scroll_shadow.svelte";

  const { data }: { data: PageData } = $props();
  const menu = data.menu;
  const iconCache = data.iconCache;

  // global keyboard events
  window.addEventListener("keydown", (ev) => {
    switch (ev.key) {
      case "ArrowDown":
        menu.selection = Math.min(menu.items.length - 1, menu.selection + 1);
        break;
      case "ArrowUp":
        menu.selection = Math.max(0, menu.selection - 1);
        break;
      case "Enter":
      case "Return":
        activateListItem(ev.altKey);
        break;
      case "Tab":
        menu.complete();
        break;
      case "Escape":
        getCurrentWindow().hide();
        break;
      default:
        // do not prevent default
        return;
    }
    // break instead of return, captured something
    ev.preventDefault();
  });

  /**
   * Activates the currently selected list item.
   * @param altKey Whether alt is pressed.
   * @param selection An index that was selected, to override the current selection.
   *                  Activates the current selection if this is not defined.
   */
  const activateListItem = (altKey: boolean, selection?: number) => {
    // in bind:group={menu.selection}, the selection is not set soon enough
    // for this to be updated correctly.
    if (selection !== undefined) {
      menu.selection = selection;
    }
    if (altKey) {
      menu.altActivate();
    } else {
      menu.activate();
    }
  };

  // query on input change
  $effect(() => {
    void commands.query(menu.inputText);
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
  listen("tauri://focus", () => {
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
      getCurrentWindow().hide();
    }
  };
</script>

<svelte:document bind:activeElement />

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
      </div>
      <ScrollShadow>
        <div class="list">
          {#each menu.items as { id, description, title, icon }, i (id)}
            <label class="list-item">
              <input
                class="list-item-radio"
                type="radio"
                name="result-list"
                value={i}
                bind:group={menu.selection}
                onclick={(e) => activateListItem(e.altKey, i)}
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
                    </div>
                  {/await}
                {/if}
              </div>
              <p class="title"><strong>{title}</strong></p>
              <p class="description"><span>{description}</span> ({id})</p>
            </label>
          {/each}
        </div>
      </ScrollShadow>
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
      background-image: url("background.jpg");
      background-size: cover;
      filter: blur(2vw);
    }
  }

  .menu {
    background: var(--color-surface);
    opacity: 0.93;

    width: 800px;
    max-height: 600px;
    display: grid;
    grid-template-rows: auto 1fr;
  }

  .search-bar {
    font-size: var(--fs-large);
    padding: 2rem;
  }

  .search-input {
    width: 100%;
    color: var(--color-on-surface);

    &::placeholder {
      color: var(--color-on-surface-variant);
    }
  }

  .list {
    display: grid;
    gap: 1rem;
    padding: 1rem;

    &:empty {
      display: none;
    }
  }

  .list-item {
    --_row-gap: 0.5rem;

    padding: 1rem;
    border-radius: var(--brad-standard);

    display: grid;
    grid-template-areas: "icon title" "icon description";
    grid-template-columns: auto 1fr;
    row-gap: var(--_row-gap);

    .icon {
      // make it take up the same size as a list item
      // with one row for the title + description
      --_icon-size: calc(
        (var(--fs-standard) + var(--fs-small)) * var(--line-height) +
          var(--_row-gap)
      );

      grid-area: icon;
      display: grid;
      place-items: center;
      width: var(--_icon-size);
      // needs to be a margin here instead of column-gap
      // so that no icon doesn't add a column
      margin-right: 1rem;

      .icon-img {
        width: var(--_icon-size);
      }

      .icon-text {
        font-size: calc(var(--_icon-size) / var(--line-height));
      }

      &:empty {
        display: none;
      }
    }

    .title {
      grid-area: title;
    }

    .description {
      grid-area: description;
      font-size: var(--fs-small);
      color: var(--color-on-surface-variant);
    }

    &:hover {
      background: var(--color-surface-bright);
    }

    &:has(.list-item-radio:checked) {
      background: var(--color-primary-container);
      // description should have this colour too
      * {
        color: var(--color-on-primary-container);
      }
    }
  }

  .list-item-radio {
    display: none;
  }

  .positioner {
    display: grid;
    place-items: center;
    width: 100vw;
    height: 100vh;
  }

  // text in the menu should not be selectable
  .menu-wrapper * {
    user-select: none;
  }
</style>
