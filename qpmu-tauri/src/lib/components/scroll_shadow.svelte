<script lang="ts">
  import type { Snippet } from "svelte";

  let { children }: { children: Snippet } = $props();

  let distanceFromTop = $state(0);
  let distanceFromBottom = $state(0);

  let topOpacity = $derived(`${Math.min(distanceFromTop / 2, 100)}%`);
  let bottomOpacity = $derived(`${Math.min(distanceFromBottom / 2, 100)}%`);

  const scrollHandler = (ev: Event) => {
    const el = ev.currentTarget! as HTMLElement;
    distanceFromTop = el.scrollTop;
    distanceFromBottom = el.scrollHeight - el.clientHeight - el.scrollTop;
  };
</script>

<div class="scroll-wrapper">
  <div class="scroll-with-shadows" onscroll={scrollHandler}>
    <div class="shadow top" style:opacity={topOpacity}></div>
    {@render children()}
    <div class="shadow bottom" style:opacity={bottomOpacity}></div>
  </div>
</div>

<style lang="scss">
  .scroll-wrapper {
    position: relative;
    display: grid;
  }

  .scroll-with-shadows {
    overflow: auto;
  }

  .shadow {
    position: absolute;
    left: 0;
    right: 0;
    height: 2rem;
    pointer-events: none;
  }

  // sometimes theres a pixel that's not covered by the gradient
  // and it looks really weird. add -1px to make sure it's covered.
  .top {
    top: -1px;
    background: radial-gradient(
      farthest-side at 50% 0,
      rgba(0 0 0 / 0.3),
      rgba(0 0 0 / 0) 100%
    );
  }

  .bottom {
    bottom: -1px;
    background: radial-gradient(
      farthest-side at 50% 100%,
      rgba(0 0 0 / 0.3),
      rgba(0 0 0 / 0) 100%
    );
  }
</style>
