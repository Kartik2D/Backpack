<script>
  let {
    value = $bindable(""),
    placeholder = "",
    disabled = false,
    class: className = "",
    ...rest
  } = $props();
</script>

<input
  class="input {className}"
  bind:value
  {placeholder}
  {disabled}
  {...rest}
/>

<style>
  .input {
    width: 100%;
    min-width: 0;
    padding: var(--space-3) var(--space-4);
    border: var(--bw) solid var(--c-border);
    border-radius: var(--radius-md);
    background: var(--c-surface-2);
    color: var(--c-text);
    font: inherit;
    font-size: var(--font-md);
    outline: none;
    transition: border-color 0.12s, box-shadow 0.12s;
  }

  .input:focus {
    border-color: var(--c-accent);
    box-shadow: 0 0 0 var(--bw) var(--c-accent);
  }

  .input:disabled {
    opacity: 0.55;
    cursor: default;
  }

  .input::placeholder {
    color: var(--c-text-muted);
  }
</style>
