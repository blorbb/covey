<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import * as commands from "../commands";
  import { Menu } from "../setup.svelte";

  const observer = new ResizeObserver(() => {
    const width = document.documentElement.clientWidth;
    const height = document.body.scrollHeight;
    console.log(width, height);
    void invoke("set_window_size", {
      width,
      height,
    });
  });
  observer.observe(document.documentElement);

  let menu = new Menu();

  $effect(() => {
    void commands.query(menu.inputText);
  });

  let mainInput = $state<HTMLInputElement>();
  $effect(() => {
    mainInput?.setSelectionRange(menu.textSelection[0], menu.textSelection[1]);
  });
</script>

<main>
  <div class="input">
    <input type="text" bind:value={menu.inputText} bind:this={mainInput} />
  </div>

  <div class="scroller">
    {#each menu.items as { id, description, title }, i (id)}
      <label>
        <input
          type="radio"
          name="result-list"
          value={i}
          bind:group={menu.selection}
        />
        <p><strong>{title}</strong></p>
        <p><span>{description}</span> ({id})</p>
      </label>
    {/each}
  </div>
</main>

<style>
  .scroller {
    max-height: 600px;
  }

  :global(body) {
    padding: 0;
    margin: 0;
  }

  :root {
    box-sizing: border-box;

    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;

    color: #0f0f0f;
    background-color: #f6f6f6;

    font-synthesis: none;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    -webkit-text-size-adjust: 100%;

    overflow: hidden;
  }

  *,
  *::before,
  *::after {
    box-sizing: inherit;
    padding: 0;
    margin: 0;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      color: #f6f6f6;
      background-color: #2f2f2f;
    }
  }
</style>
