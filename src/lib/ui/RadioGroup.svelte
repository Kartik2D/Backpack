<script>
  import Button from "$lib/ui/Button.svelte";
  import Stack from "$lib/ui/Stack.svelte";

  let {
    options = /** @type {{ value: string; label: string }[]} */ ([]),
    value = $bindable(""),
    class: className = "",
  } = $props();
</script>

<Stack gap={1} class="radio-group {className}">
  {#each options as option (option.value)}
    <Button
      variant={value === option.value ? "primary" : "default"}
      full
      align="start"
      onclick={() => (value = option.value)}
    >
      {option.label}
    </Button>
  {/each}
</Stack>
