<script lang="ts" generics="T">
  import type { Snippet } from "svelte";

  let {
    items = $bindable(),
    item,
    key,
  }: {
    items: T[];
    item: Snippet<[T]>;
    key: (item: T) => unknown;
  } = $props();
</script>

<ol class="dnd-list">
  {#each items as i (key(i))}
    <li>
      {@render item(i)}
    </li>
  {/each}
</ol>

<style lang="scss">
  li {
    list-style-type: none;
  }
</style>
