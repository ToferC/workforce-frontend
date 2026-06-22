# Phase 2 — Tokens, typography & colour (implementation notes)

**Plan:** [GC Design System & accessibility compliance plan](../GC_DESIGN_SYSTEM_COMPLIANCE.md)
**Scope:** CSS/formatting only. No business logic, data, HTMX, or template
structure changed — the work is confined to `static/css/theme.css`, which every
Bootstrap-rendered page consumes through its CSS custom properties.

## What landed

### `theme.css` now consumes GCDS tokens
The existing custom-property *names* were kept (so every `var(--…)` reference
across `theme.css` and the templates keeps working), but their *values* now
resolve to GC Design System tokens from `gcds.css`:

- **Light is the default theme** (`:root` / `[data-theme="light"]`), matching the
  GC baseline; dark mode moved to an opt-in `[data-theme="dark"]` block. This
  aligns with `base.html` defaulting to light in Phase 1.
- **Colour** maps onto GCDS tokens: backgrounds → `--gcds-bg-white/light` +
  grayscale; text → grayscale 800/700/600 (all AA on white); links →
  `--gcds-link-default` (#1f497a) / `--gcds-link-hover` (#1354ec); success →
  green-750, danger → red-600, etc.
- **Spacing** (`--spacing-xs…xl`) maps onto the `--gcds-spacing-*` scale;
  **radii** onto `--gcds-border-radius-sm/md`.
- **Dark mode** retuned so each pair clears WCAG AA: lightened secondary/muted
  text, blue-300 links, green-350 success, red-300 danger; the neon-green code
  text (`#00ff41`) is gone in both themes.

### GC typography
- `body` → `--gcds-font-families-body` (**Noto Sans**), at 1.0625rem — slightly
  below the 1.25rem GC default so this app's dense data tables don't overflow,
  while still more readable than the previous 16px system font.
- Headings → `--gcds-font-families-heading` (**Lato**).
- Code/`pre` → `--gcds-font-families-monospace` (**Noto Sans Mono**), replacing
  the old Monaco/Menlo stack.

### Domain chips re-derived from the GCDS palette
The hard-coded chip hexes (`viz.html` colours) are gone. Light (default) uses a
soft tint + dark accent text; dark mode uses a deep solid fill + white text —
each pair chosen to clear AA. Capability pips and status chips already read from
the colour variables, so they pick up the new palette automatically.

### Reduced motion
Added a `prefers-reduced-motion: reduce` block that neutralizes the global
transitions, hover transforms, and smooth scroll (WCAG 2.3.3).

## Deliberately left for later (out of "formatting-only" scope)
- **ECharts series colours** still come from handler-aggregated JSON, not CSS
  variables. Re-theming charts touches handler code, so it's deferred (plan
  Phase 2 chart task / a later pass) to honour "keep business logic and data the
  same."
- **Heatmap cell text colour** in `analytics/coverage.html` (`#fff` / `#666`)
  is computed inline from data opacity — left as-is since it's tied to the
  data rendering.
- Bootstrap remains loaded; its components are restyled via the shared variables
  but the framework itself is removed in Phase 5.

## Validation
- All `var(--gcds-…)` references in `theme.css` verified to resolve against
  `gcds.css`; brace balance checked.
- Visual rendering and the AA contrast pass still need a running dev server +
  the `a11y/` harness (no DB/API in the authoring environment) — carried with
  the Phase 1 live-baseline task.

## Manual check when a dev server is up
- [ ] Pages render in Lato (headings) / Noto Sans (body) with the GC palette.
- [ ] Light is default; dark toggle still works and is AA-legible.
- [ ] Domain chips, capability pips, tables, cards, buttons look on-palette.
- [ ] `npm run a11y` shows no new contrast violations vs. the baseline.
