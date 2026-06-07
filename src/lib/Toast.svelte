<script>
  import { ToastItem, ToastViewport } from "$lib/ui/index.ts";
  import { toastState, toasts } from "$lib/toast.svelte.ts";
</script>

<ToastViewport>
  {#each toastState.items as toast (toast.id)}
    <ToastItem
      type={toast.type}
      message={toast.message}
      onDismiss={toast.type === "loading" ? undefined : () => toasts.dismiss(toast.id)}
    />
  {/each}
</ToastViewport>
