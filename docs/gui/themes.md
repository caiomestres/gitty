# Themes

Gitty includes three visual themes: **Default** (warm cream), **Dark**, and **World Cup - Brasil**. Themes change all design tokens — colors, typography, and visual personality.

## Available Themes

### Default

The classic Gitty appearance inspired by Cursor IDE's warm aesthetic.

```
┌─────────────────────────────┐
│                             │
│         🦁                  │
│                             │
│      Default                │
│                             │
│    Warm cream canvas        │
│    Cursor Orange accent     │
│    Inter + JetBrains Mono   │
│                             │
└─────────────────────────────┘
```

**Characteristics:**
- Warm off-white background (`#f7f7f4`)
- Cursor Orange primary (`#f54e00`)
- Dark ink text (`#26251e`)
- Golden lion tamarin mascot
- Clean, editorial aesthetic

### Dark

Eye-friendly dark mode for low-light environments.

```
┌─────────────────────────────┐
│                             │
│         🦁                  │
│                             │
│       Dark                  │
│                             │
│    Dark navy canvas         │
│    Luminous orange accent   │
│    High contrast text       │
│                             │
└─────────────────────────────┘
```

**Characteristics:**
- Dark navy background (`#1a1a2e`)
- Brighter orange for visibility (`#ff6b35`)
- Light text on dark (`#f0f0f0`)
- Luminous mascot variant
- Reduced eye strain

### World Cup - Brasil

Celebrate with national colors.

```
┌─────────────────────────────┐
│                             │
│         🦁                  │
│                             │
│   World Cup - Brasil        │
│                             │
│    Green, Yellow, Blue      │
│    National palette         │
│    Festive mascot           │
│                             │
└─────────────────────────────┘
```

**Characteristics:**
- Green primary (`#009c3b`)
- Yellow accents (`#ffdf00`)
- Blue highlights (`#002776`)
- Cream background variant
- Brasil-themed mascot colors

## Switching Themes

### Settings Page

Navigate to **Settings** → **Theme**:

```
Theme
════════

Select a theme:

┌──────────────────────────────────────────────────────┐
│ ┌────────────┐  ┌────────────┐  ┌────────────┐      │
│ │  🦁        │  │  🦁        │  │  🦁        │      │
│ │  Default   │  │   Dark     │  │   Brasil   │      │
│ │  ✓         │  │            │  │            │      │
│ └────────────┘  └────────────┘  └────────────┘      │
└──────────────────────────────────────────────────────┘
```

Click any theme card to switch. Changes apply immediately.

### Bottom Bar Toggle

Quick switch from any page:

```
┌─────────────────────────────────────────────────────────┐
│ [🎨 Theme ▼]  [🔔]  15 repos  [⏰]                    │
│     ├─ Default                                           │
│     ├─ Dark                                              │
│     └─ World Cup - Brasil                                │
└─────────────────────────────────────────────────────────┘
```

Click the theme button, select new theme. Returns to previous page.

### CLI

```bash
# Set theme
gitty config set theme dark

# Get current
gitty config get theme

# List available
gitty config themes
```

## Theme Details

### What Changes

| Element | Themed? |
|---------|---------|
| Background colors | ✓ |
| Text colors | ✓ |
| Primary/accent colors | ✓ |
| Status colors | ✓ (mapped to theme palette) |
| Border colors | ✓ |
| Mascot colors | ✓ |
| Font families | ✗ (always Inter + JetBrains Mono) |
| Font sizes | ✗ (consistent across themes) |
| Spacing | ✗ (layout remains constant) |
| Border radius | ✗ (component structure preserved) |

### Token Coverage

Each theme defines all CSS custom properties:

```css
/* Colors */
--color-primary
--color-primary-active
--color-ink
--color-body
--color-muted
--color-canvas
--color-surface-card
/* ... all color tokens ... */

/* Mascot-specific */
--mascot-bg
--mascot-mane-primary
--mascot-mane-secondary
--mascot-face
--mascot-eye
```

See `src/lib/styles/theme-*.css` for complete definitions.

### Design Token Override

Themes use the `[data-theme]` attribute selector:

```css
/* Default theme (no attribute or default) */
:root {
  --color-primary: #f54e00;
}

/* Dark theme */
[data-theme="dark"] {
  --color-primary: #ff6b35;
}

/* Brasil theme */
[data-theme="world-cup-brasil"] {
  --color-primary: #009c3b;
}
```

The `data-theme` attribute on `<html>` activates the theme.

## Per-Theme Mascot

The mascot changes colors with each theme:

| Theme | Mascot Style |
|-------|--------------|
| **Default** | Golden/orange fur, warm background |
| **Dark** | Luminous/cream fur, dark canvas |
| **Brasil** | Green/yellow/blue national colors |

**Guarantee:** Same silhouette, same character pose — only colors change.

## Custom Themes

Custom themes are **not supported** in v1. The theme system is designed for extension:

```
Future: User-defined themes
• Custom CSS file import
• Color picker for key tokens
• Theme marketplace (?)
```

Currently, only the three bundled themes are available.

## Best Practices

### Choosing a Theme

| Scenario | Recommended Theme |
|----------|-------------------|
| Daily work | Default or Dark (preference) |
| Bright room | Default |
| Dark room | Dark |
| Eye strain | Dark |
| Screenshots | Default (most recognizable) |
| Brasil match days | World Cup - Brasil |

### Switching

1. **Preview in Settings** — See all themes before committing
2. **Use bottom bar** — Quick toggle for temporary changes
3. **Persist preference** — Automatically saved to config

### Accessibility

All themes maintain:

- **WCAG AA contrast** — Text readable at all sizes
- **Status distinguishability** — Success/Warning/Error colors distinct
- **Focus indicators** — Visible keyboard focus
- **Motion respect** — Respects `prefers-reduced-motion`

## Troubleshooting

### Theme not applying

1. Click the theme card (not hover)
2. Check browser console for CSS errors
3. Verify `data-theme` attribute on `<html>`
4. Try restarting the app

### Colors look wrong

1. Check for custom user styles interfering
2. Verify theme CSS loaded (network tab)
3. Ensure no browser extensions modifying colors

### Flash of wrong theme

On startup, brief flash before theme applies:

- **Cause:** Theme loaded after first paint
- **Mitigation:** Theme applied as early as possible
- **Workaround:** Normal behavior, usually sub-second

### Mascot not changing

1. Mascot uses CSS variables (check CSS is loaded)
2. Verify mascot component has `mascot-svg` class
3. Check browser devtools for variable values

## Technical Details

### Storage

Theme preference stored in config:

```json
{
  "theme": "dark"
}
```

Values: `"default"`, `"dark"`, `"world-cup-brasil"`

### Application

1. App starts
2. Config loaded
3. Theme value extracted
4. `data-theme` attribute set on `<html>`
5. CSS variables apply automatically

### Fallback

Invalid theme values fall back to `"default"`:

```javascript
const theme = config.theme || 'default';
const validThemes = ['default', 'dark', 'world-cup-brasil'];
const appliedTheme = validThemes.includes(theme) ? theme : 'default';
```

## See Also

- [Settings](settings.md) — Theme configuration
- [Brand Identity](../concepts/domain.md#theme) — Design system
- Custom CSS: Edit `src/lib/styles/theme-*.css` (advanced)