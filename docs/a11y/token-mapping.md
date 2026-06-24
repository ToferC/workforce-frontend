# Phase 0 — GCDS token integration decision

**Plan:** [GC Design System & accessibility compliance plan](../GC_DESIGN_SYSTEM_COMPLIANCE.md)

Decision record for how the existing CSS custom properties in
`static/css/theme.css` map onto `@cdssnc/gcds-tokens`. The goal: keep the
existing variable *names* (so the dozens of `var(--…)` references across
`theme.css` and templates keep working) but repoint their *values* at GCDS
tokens. This lets Phase 2 swap the palette without touching every consumer.

## Approach

1. Vendor `@cdssnc/gcds-tokens` into `static/gcds/` and load its CSS before
   `theme.css` in `base.html`.
2. In `theme.css`, set each existing variable to the corresponding GCDS token
   (`var(--gcds-…)`) instead of a raw hex.
3. Remove raw hex values from templates/macros (F7) and have them consume the
   mapped variables.
4. Make **light** the default theme; treat dark as an opt-in whose token pairs
   are each AA-verified (F9).

> Token variable names below follow the GCDS naming convention. Confirm exact
> names/values against the vendored `gcds-tokens` package at implementation time
> — these are the intended mappings, not copy-paste-final values.

## Proposed mapping (light/default theme)

| Existing variable (`theme.css`) | GCDS token intent | Notes |
|---|---|---|
| `--text-primary` | `--gcds-text-primary` (`#333`) | Body text, AA on white. |
| `--text-secondary` | `--gcds-text-secondary` | Muted/supporting text — re-check AA. |
| `--text-muted` | `--gcds-text-secondary` | Collapse `muted` into a token that passes AA (current `#868e96` is borderline on white). |
| `--bg-primary` | `--gcds-bg-default` (`#fff`) | Page/surface background. |
| `--bg-secondary` / `--bg-tertiary` | `--gcds-bg-light` / surface tokens | Stripes, card headers. |
| `--color-primary` (link/action) | `--gcds-link-default` (`#284162`) | Link + primary action. |
| `--color-primary-hover` | `--gcds-link-hover` (`#0535d2`) | |
| Focus outline (`--color-primary`) | `--gcds-focus` / link-focus token | Standardize the `:focus-visible` outline on the GCDS focus token. |
| `--color-success` | GCDS feedback "success" | Notices/buttons. |
| `--color-danger` | GCDS feedback "danger"/error | Error text must hit ≥4.5:1. |
| `--color-warning` | GCDS feedback "warning" | Warning text on light needs dark foreground. |
| `--color-info` | GCDS feedback "info" | |
| `--border-color` / `--border-color-light` | `--gcds-border-default` / light | Table/card/input borders. |
| `--spacing-xs…xl` | `--gcds-spacing-*` (8px scale) | Re-map to the GCDS spacing steps; keep names. |
| `--radius-sm…lg` | GCDS border-radius tokens | GCDS uses restrained radii. |
| Font stack (`body`) | `--gcds-font-families-body` (Lato → Noto Sans) | Plus base size/line-height from GCDS type tokens. |
| `--code-text` `#00ff41` | a token-based code colour | Replace neon green; must pass AA on the code background. |
| Domain chip hexes (`viz.html`) | derive from GCDS palette | AA-checked foreground/background pairs per chip, both themes. |

## Dark theme (opt-in)

Keep the `[data-theme="dark"]` block but every pair must pass AA:
- Re-verify `--text-secondary`/`--text-muted` on `--bg-primary` (`#1a1d23`).
- Replace `--code-text: #00ff41` and `--color-danger: #ff4444` (low contrast on
  dark) with AA-passing values.
- Ensure ECharts series colours (read from CSS vars) remain ≥3:1 against the
  dark surface after the `themechange` event.

If AA cannot be guaranteed for dark mode within Phase 2, ship light-only and
defer dark mode to a follow-up rather than ship a non-compliant default.

## Acceptance for closing Phase 0

- [x] Audit harness committed and documented (`a11y/`).
- [x] Static baseline inventory committed (`baseline-inventory.md`).
- [x] Token integration approach decided (this doc).
- [ ] Live axe/pa11y numeric baseline captured once a dev server is available
      (carried into Phase 1 as the first task).

## Update — implemented in Phase 2

This mapping was applied in `static/css/theme.css`; see
[`phase-2-notes.md`](./phase-2-notes.md). Note the final implementation uses the
GCDS **primitive** tokens that `gcds.css` actually exposes (e.g.
`--gcds-color-grayscale-800`, `--gcds-link-default`, `--gcds-bg-white`,
`--gcds-color-green-750`) rather than the semantic `--gcds-text-*` names sketched
above — this package version ships the primitive scale, so the table's "intent"
column was realized with the nearest AA-passing primitive.
