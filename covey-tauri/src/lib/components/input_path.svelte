<script lang="ts">
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
  <button class="input-path-picker" onclick={onPick}> {buttonText} </button>
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
    background: var(--color-secondary-container);
    color: var(--color-on-secondary-container);
    border-radius: 999rem;
    padding: 0.25rem 0.5rem;

    transition: var(--time-transition) filter;
    &:hover {
      filter: brightness(1.2) var(--filter-shadow-large);
    }
  }
</style>
