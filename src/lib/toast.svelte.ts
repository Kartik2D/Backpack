type ToastType = "loading" | "success" | "error";

type Toast = {
  id: number;
  type: ToastType;
  message: string;
};

let nextId = 1;

export const toastState = $state({
  items: [] as Toast[],
});

function remove(id: number) {
  toastState.items = toastState.items.filter((toast) => toast.id !== id);
}

function add(type: ToastType, message: string, timeout = 3000) {
  const id = nextId++;
  toastState.items = [...toastState.items, { id, type, message }];

  if (timeout > 0) {
    window.setTimeout(() => remove(id), timeout);
  }

  return id;
}

export const toasts = {
  loading(message: string) {
    return add("loading", message, 0);
  },
  success(message: string) {
    return add("success", message);
  },
  error(message: string) {
    return add("error", message, 4500);
  },
  dismiss: remove,
};
