<script lang="ts">
  let { margin = "0px" }: { margin?: string } = $props();
</script>

<div class="divider" style:margin-block={margin}></div>

<style lang="scss">
  .divider {
    height: 1px;
    background-color: var(--color-outline-variant);
  }
</style>
