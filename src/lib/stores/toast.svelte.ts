export interface Toast {
  id: string;
  message: string;
  hint?: string;
  severity: "error" | "warning" | "info";
  dismissAfterMs: number | null;
}

let toasts = $state<Toast[]>([]);
let nextId = 0;

export function getToasts(): Toast[] {
  return toasts;
}

export function addToast(toast: Omit<Toast, "id">): string {
  const id = `toast-${++nextId}`;
  const newToast: Toast = { ...toast, id };
  toasts = [...toasts, newToast].slice(-3);

  if (toast.dismissAfterMs !== null) {
    setTimeout(() => dismissToast(id), toast.dismissAfterMs);
  }

  return id;
}

export function dismissToast(id: string): void {
  toasts = toasts.filter((t) => t.id !== id);
}
