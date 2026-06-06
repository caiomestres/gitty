import { addToast } from "$lib/stores/toast.svelte";

export interface HandledError {
  message: string;
  hint?: string;
}

interface ErrorLike {
  code?: string;
  message?: string;
  hint?: string;
  transient?: boolean;
}

/**
 * Classifies and routes errors: transient errors go to toast and return null,
 * non-transient errors are returned for display in-page.
 */
export function handleError(err: unknown): HandledError | null {
  let message: string;
  let hint: string | undefined;
  let transient = false;

  if (typeof err === "string") {
    message = err;
  } else if (err && typeof err === "object") {
    const e = err as ErrorLike;
    message = e.message ?? String(err);
    hint = e.hint;
    transient = e.transient === true;
  } else {
    message = String(err);
  }

  if (transient) {
    addToast({ message, hint, severity: "error", dismissAfterMs: 5000 });
    return null;
  }

  return { message, hint };
}
