<script lang="ts">
  import Button from "./button.svelte";

  let {
    value,
    onSetValue,
    onPick,
    buttonText,
  }: {
    value?: string;
    onSetValue: (path: string) => void;
    onPick: () => void;
    buttonText: string;
  } = $props();
</script>

<!-- 
  @component

  Generic file or folder path picker input element.
  Use `InputFilePath` or `InputFolderPath` instead.
-->

<div class="input-path-wrapper">
  <input
    type="text"
    class="input-path"
    {value}
    onchange={(e) => onSetValue(e.currentTarget.value)}
    onfocusout={(e) =>
      (e.currentTarget.scrollLeft = e.currentTarget.scrollWidth)}
  />
  <div class="input-path-picker">
    <Button theme="secondary" pill onclick={onPick}>{buttonText}</Button>
  </div>
</div>

<style lang="scss">
  .input-path-wrapper {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 0.5rem;
  }

  .input-path {
    border-bottom: 2px solid var(--color-outline);
    background: var(--color-surface-container);
    padding: 0.25rem 0.5rem;
  }

  .input-path-picker {
    display: grid;
  }
</style>
