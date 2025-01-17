<script lang="ts">
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import * as commands from "../commands";
  import { Menu } from "../setup.svelte";
  import { onDestroy } from "svelte";

  let menu = new Menu();

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
      default:
        // do not prevent default
        return;
    }
    // break instead of return, captured something
    ev.preventDefault();
  });

  const activateListItem = (altKey: boolean) => {
    if (altKey) {
      menu.altActivate();
    } else {
      menu.activate();
    }
  };

  $effect(() => {
    void commands.query(menu.inputText);
  });

  let mainInput = $state<HTMLInputElement>();
  $effect(() => {
    mainInput?.focus();
    mainInput?.setSelectionRange(menu.textSelection[0], menu.textSelection[1]);
  });

  // needs to be put on the onpointerdown event
  // just being on the onblur doesn't work for some reason
  const refocusInput = (ev: Event) => {
    ev.preventDefault();
    mainInput?.focus();
  };

  let unlisten: UnlistenFn | undefined;
  listen("tauri://focus", () => {
    mainInput?.focus();
    mainInput?.setSelectionRange(0, mainInput.value.length);
  }).then((f) => (unlisten = f));

  onDestroy(() => unlisten?.());
</script>

<div
  class="positioner"
  onpointerdown={(ev) => {
    // allow clicks inside the input to work
    if (ev.target !== mainInput) {
      refocusInput(ev);
    }
  }}
>
  <div class="menu-wrapper">
    <main class="menu">
      <div class="search-bar">
        <input
          class="search-input"
          type="text"
          bind:value={menu.inputText}
          bind:this={mainInput}
          placeholder="Search..."
          onblur={refocusInput}
        />
      </div>
      <div class="list-scroller">
        <div class="list">
          {#each menu.items as { id, description, title }, i (id)}
            <label class="list-item">
              <input
                class="list-item-radio"
                type="radio"
                name="result-list"
                value={i}
                bind:group={menu.selection}
                onclick={(e) => activateListItem(e.altKey)}
              />
              <p class="title"><strong>{title}</strong></p>
              <p class="description"><span>{description}</span> ({id})</p>
            </label>
          {/each}
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
      background-image: url("background.jpg");
      background-size: cover;
      filter: blur(2vw);
    }
  }

  .menu {
    background: var(--color-surface);
    opacity: 0.93;
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

  .list-scroller {
    max-height: 500px;
    overflow: auto;
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
    padding: 1rem;
    border-radius: var(--brad-standard);

    display: grid;
    gap: 4px;

    .description {
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
    align-items: center;
    width: 100vw;
    height: 100vh;
  }

  // text in the menu should not be selectable
  .menu-wrapper * {
    user-select: none;
  }
</style>
