# M6 Polish — Design

## 1. Self-Hosted Fonts

### Font Files

Download from Google Fonts (woff2 format only — best compression, universal browser support):

```
static/fonts/
├── inter-400.woff2
├── inter-500.woff2
├── inter-600.woff2
├── jetbrains-mono-400.woff2
└── jetbrains-mono-500.woff2
```

### CSS Changes

Replace the `@import url(...)` in `global.css` with `@font-face` declarations:

```css
@font-face {
  font-family: "Inter";
  font-weight: 400;
  font-style: normal;
  font-display: swap;
  src: url("/fonts/inter-400.woff2") format("woff2");
}
/* ... repeat for 500, 600, and JetBrains Mono variants */
```

### CSP

The existing CSP in `tauri.conf.json` already allows `'self'` for all resource types. Self-hosted fonts in `static/` are served from `'self'`, so no CSP change is needed.

---

## 2. Design Token Completion

### New Tokens for `tokens.css`

Add the missing tokens from DESIGN.md:

```css
:root {
  /* Missing spacing */
  --space-section: 80px;

  /* Missing semantic color */
  --color-warning: #c08532;

  /* Typography composites — utility classes in global.css */
}
```

### Typography Utility Classes for `global.css`

Add classes that compose the DESIGN.md typography tokens:

```css
.text-display-lg { font-size: 36px; font-weight: 400; line-height: 1.2; letter-spacing: -0.72px; }
.text-display-md { font-size: 26px; font-weight: 400; line-height: 1.25; letter-spacing: -0.325px; }
.text-display-sm { font-size: 22px; font-weight: 400; line-height: 1.3; letter-spacing: -0.11px; }
.text-title-md   { font-size: 18px; font-weight: 600; line-height: 1.4; }
.text-title-sm   { font-size: 16px; font-weight: 600; line-height: 1.4; }
.text-body-md    { font-size: 16px; font-weight: 400; line-height: 1.5; }
.text-body-sm    { font-size: 14px; font-weight: 400; line-height: 1.5; }
.text-caption    { font-size: 13px; font-weight: 400; line-height: 1.4; }
.text-caption-up { font-size: 11px; font-weight: 600; line-height: 1.4; letter-spacing: 0.88px; text-transform: uppercase; }
```

### Component Audit Strategy

Systematic pass through each `*.svelte` file:
1. Replace hardcoded hex colors with `var(--color-*)` references
2. Replace hardcoded pixel values for padding/margin/gap with `var(--space-*)` where a matching token exists
3. Replace `font-size: Npx` with the appropriate typography class or token reference
4. Leave values that don't map to any token (e.g., `2px` border widths, `max-width: 280px`) as documented exceptions

---

## 3. Toast Error System

### Architecture

A new `ToastStore` (Svelte 5 runes) + `ToastContainer` component mounted at the `AppShell` level (viewport-scoped, persists across page navigation).

```
AppShell.svelte
├── ToastContainer.svelte (fixed position, top-right, z-index: 200)
│   └── Toast.svelte × N (max 3 visible)
├── Sidebar
└── <slot /> (page content — inline errors live here)
```

### Toast Store

```typescript
// src/lib/stores/toast.ts
interface Toast {
  id: string;
  message: string;
  hint?: string;
  severity: "error" | "warning" | "info";
  dismissAfterMs: number | null; // null = manual dismiss only
}

// Svelte 5 runes-based store
let toasts = $state<Toast[]>([]);

export function addToast(toast: Omit<Toast, "id">): void { ... }
export function dismissToast(id: string): void { ... }
```

### Error Classification → Toast vs Inline

The frontend `errorMessage()` helper is replaced with a smarter `handleError()`:

```typescript
export function handleError(err: unknown): { message: string; hint?: string; isTransient: boolean } {
  // Parse AppError from Tauri invoke rejection
  // Transient: git_error (Network), lock_contention
  // Persistent: config_error, io_error, git_error (non-Network), everything else
}
```

Pages call `handleError()` and route to either `addToast()` (transient) or set local `error` state (persistent).

### Toast Styling

Per DESIGN.md:
- Background: `var(--color-surface-card)` with `var(--color-hairline)` border
- Error icon: `var(--color-error)` dot
- Text: `var(--color-body)` message + `var(--color-muted)` hint
- Dismiss: `btn-icon` pattern
- Border-radius: `var(--radius-lg)`
- Width: 360px max
- Position: fixed top-right with `var(--space-base)` offset

---

## 4. AppError Recovery Hints

### Backend Changes

Add a `hint` field to `AppError` in `src-tauri/src/error.rs`:

```rust
pub struct AppError {
    pub code: String,
    pub message: String,
    pub hint: Option<String>,
}
```

Populate `hint` in the `From<CoreError>` implementation based on variant:

| CoreError Variant | Hint |
|---|---|
| `GitNotFound` | "Install Git and ensure it is in your PATH, then restart Gitty." |
| `LockContention` | "Another process is using this Repository. Wait for it to finish or check for stale locks." |
| `UnsupportedSchema` | "This config was created by a newer version of Gitty. Update Gitty to the latest version." |
| `PathNotFound` | "The path does not exist on disk. Check that it hasn't been moved or deleted." |
| `Io` (permission denied) | "Permission denied. Check file permissions or run with elevated privileges." |
| `NoConfigDir` | "Could not find a config directory. Check your OS user profile." |
| All others | `None` |

### Frontend Changes

Update `ErrorDto` in `workspace.ts`:

```typescript
export interface ErrorDto {
  code: string;
  message: string;
  hint?: string;
}
```

The `handleError()` function extracts `hint` for display.

---

## 5. Macro-Level Retry

### Core Domain Changes

Add `RetryConfig` to `Step` in `macro_def.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,       // 1-10, default 3
    #[serde(default = "default_backoff")]
    pub backoff_seconds: u64,    // base backoff, default 2
}

fn default_backoff() -> u64 { 2 }
```

Add to `Step`:

```rust
pub struct Step {
    pub kind: StepKind,
    pub condition: Option<String>,
    pub rollback: Option<Box<Step>>,
    pub confirm: bool,
    #[serde(default)]
    pub retry: Option<RetryConfig>,
}
```

### Execution Engine Changes

In `execution.rs`, `execute_git_op` gains retry logic:

```
fn execute_git_op_with_retry(step_index, op, repo, git, retry_config) -> StepResult:
    for attempt in 0..max_attempts:
        result = execute_git_op(step_index, op, repo, git)
        if result.is_success OR error is not Network:
            return result
        if attempt < max_attempts - 1:
            sleep(backoff_seconds * 2^attempt, capped at 60s)
    return last_result
```

Shell Command steps ignore `retry` — the execution engine checks `step.kind` before applying retry.

### CLI Syntax

Extend step parsing (D20) in `gitty-cli`:

```
fetch:retry=3           → RetryConfig { max_attempts: 3, backoff_seconds: 2 }
pull:retry=3:backoff=5  → RetryConfig { max_attempts: 3, backoff_seconds: 5 }
fetch                   → No retry (current behavior)
```

### Config Schema

No schema bump needed — `#[serde(default)]` on the `retry` field means existing configs load fine with `retry: None`.

### GUI Changes

In `MacroEditor.svelte`, Git Op steps get an optional "Retry on network error" section:
- Checkbox: "Retry on network error" (toggles `retry` field)
- When checked: number input for max attempts (1-10, default 3)

---

## Data Flow Summary

```
User action → invoke("run_macro") → Tauri IPC
  → execute_macro() → for each step:
    if GitOp and retry configured:
      execute_git_op_with_retry(step, retry_config)
    else:
      execute_git_op(step) or execute_shell(step)
  → StepResult (includes output from last attempt)
  → JobDto → frontend
  → handleError() classifies:
    transient → addToast()
    persistent → set page error state (with hint)
```
