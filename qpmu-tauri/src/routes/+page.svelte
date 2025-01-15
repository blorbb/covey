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
        if (ev.altKey) {
          menu.altActivate();
        } else {
          menu.activate();
        }
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

  $effect(() => {
    void commands.query(menu.inputText);
  });

  let mainInput = $state<HTMLInputElement>();
  $effect(() => {
    mainInput?.focus();
    mainInput?.setSelectionRange(menu.textSelection[0], menu.textSelection[1]);
  });

  let unlisten: UnlistenFn | undefined;
  listen("tauri://focus", (e) => {
    mainInput?.focus();
    mainInput?.setSelectionRange(0, mainInput.value.length);
  }).then((f) => (unlisten = f));

  onDestroy(() => unlisten?.());
</script>

<div class="center">
  <main>
    <div class="input-wrapper">
      <input
        class="input"
        type="text"
        bind:value={menu.inputText}
        bind:this={mainInput}
        placeholder="Search..."
      />
    </div>

    {#if menu.items.length > 0}
      <div class="scroller">
        <div class="main-list">
          {#each menu.items as { id, description, title }, i (id)}
            <label class="list-item">
              <input
                class="list-item-radio"
                type="radio"
                name="result-list"
                value={i}
                bind:group={menu.selection}
              />
              <p class="title"><strong>{title}</strong></p>
              <p class="description"><span>{description}</span> ({id})</p>
            </label>
          {/each}
        </div>
      </div>
    {/if}
  </main>
</div>

<style>
  :root {
    --window-background: rgb(0 0 0);

    /* based on the rosepine theme */
    --primary-background: #191724;
    --primary: #c4a7e7;
    --text-selection: rgba(255 255 255 / 0.2);

    --pane-background: var(--primary-background);
    --pane-background: radial-gradient(
      circle 500px at var(--entry-icon-center) var(--entry-icon-center),
      color-mix(in oklab, var(--primary) 10%, var(--primary-background)) 0%,
      var(--primary-background) 100%
    );

    --hover: rgba(255 255 255 / 0.05);
    --selected: var(--primary);
    --selected-text-color: var(--primary-background);
    --selected-faded-text-color: rgba(
      from var(--selected-text-color) r g b calc(alpha * 0.8)
    );

    --text-color: rgba(255 255 255 / 0.9);
    --faded-text-color: rgb(from var(--text-color) r g b / calc(alpha * 0.8));

    --text-size: 24px;
    --entry-text-size: calc(var(--text-size) * 1.3);
    --description-text-size: calc(var(--text-size) / 1.3);

    --padding: 12px;
    --window-padding: 0px;
    --entry-padding: calc(2 * var(--padding));

    --main-brad: 24px;
    --list-item-brad: calc(var(--main-brad) - var(--padding));
    --window-brad: calc(var(--main-brad) + var(--window-padding));

    --icon-size: 48px;
    --entry-icon-size: 48px;

    --border-color: #26233a;
    --window-border: 4px;

    /* distance to center of the entry icon, relative to the main box top left */
    --entry-icon-center: calc(
      var(--entry-padding) + var(--entry-icon-size) / 2
    );

    font-size: var(--text-size);
    color: var(--text-color);
  }

  main {
    background: var(--pane-background);
    border-radius: var(--main-brad);
    border: var(--window-border) solid var(--border-color);
    overflow: hidden;
  }

  .input-wrapper {
    font-size: var(--entry-text-size);
    padding: var(--entry-padding);
  }

  .input {
    width: 100%;
    font: inherit;
    color: inherit;
    background: transparent;
    border: none;
    outline: none;

    &::placeholder {
      color: var(--faded-text-color);
    }

    &::selection {
      background: var(--text-selection);
    }
  }

  .scroller {
    max-height: 500px;
    overflow: auto;
  }

  .main-list {
    display: grid;
    gap: calc(var(--padding) / 2);
    padding: calc(var(--padding));
  }

  .list-item {
    padding: var(--padding);
    border-radius: var(--list-item-brad);

    display: grid;
    gap: 4px;

    .description {
      font-size: var(--description-text-size);
      color: var(--faded-text-color);
    }

    &:hover {
      background: var(--hover);
    }

    .list-item-radio {
      display: none;
    }

    &:has(.list-item-radio:checked) {
      background: var(--selected);
      color: var(--selected-text-color);

      /* force this color */
      * {
        color: var(--selected-text-color);
      }
    }
  }

  :global(body) {
    padding: 0;
    margin: 0;
  }

  :root {
    box-sizing: border-box;

    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-weight: 400;

    font-synthesis: none;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    -webkit-text-size-adjust: 100%;

    overflow: hidden;
  }

  .center {
    display: grid;
    align-items: center;
    height: 100vh;
  }

  *,
  *::before,
  *::after {
    box-sizing: inherit;
    padding: 0;
    margin: 0;
    user-select: none;
    -webkit-user-select: none;
  }
</style>
