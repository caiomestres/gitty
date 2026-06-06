import { listen } from "@tauri-apps/api/event";

/**
 * Subscribe to config-changed events from the Tauri backend.
 * Returns a cleanup function suitable for use in `$effect` return.
 *
 * Usage inside `$effect`:
 *   return onConfigChanged(() => reload());
 */
export function onConfigChanged(callback: () => void): () => void {
  let unlisten: (() => void) | undefined;
  listen("config-changed", callback).then((fn) => {
    unlisten = fn;
  });
  return () => unlisten?.();
}
