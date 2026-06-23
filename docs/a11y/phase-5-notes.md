# Phase 5 — Retire Bootstrap (hybrid grid) (implementation notes)

**Plan:** [GC Design System & accessibility compliance plan](../GC_DESIGN_SYSTEM_COMPLIANCE.md)
**Approach chosen:** *Hybrid* — keep Bootstrap's grid as a safety net, drop the
Bootstrap component stylesheet, reimplement the component/utility classes still
in use with GC tokens, and adopt GCDS layout for new work going forward.

## What changed

### CSS: full Bootstrap → grid-only + GC shim
`base.html` now loads **`bootstrap-grid.min.css`** instead of the full
`bootstrap.min.css`. The grid sheet provides the grid (`container`/`row`/`col-*`),
flex/display utilities, and spacing utilities (`m-*`/`p-*`) — so **no template
layout markup changed**.

A new **`static/css/gc-components.css`** reimplements, using the GCDS-token theme
variables, the Bootstrap **component and utility** classes the templates still
use (audited from the templates, by frequency):

- Text colour/alignment: `text-muted` (335×), `text-center`, `text-dark`,
  `text-end`, `text-success/danger/primary/secondary/warning/info`,
  `text-nowrap`, `text-decoration-none`.
- Backgrounds: `bg-primary/secondary/success/danger/info/warning/light/dark`
  (+ `bg-body-secondary`).
- Buttons not already in `theme.css`: `btn-sm`, `btn-secondary`,
  `btn-outline-*`, and the unscoped contextual `btn-success/danger/info/warning`
  + disabled state.
- `badge` (+ AA-contrast text on `.bg-*`), `rounded-pill`.
- Forms: `form-check*`, `form-text`, `form-select`.
- Alerts: `alert` + `alert-*` + `alert-dismissible .close` (flash messages).
- Sizing/typography/misc: `w-100`, `h-100`, `lead`, `display-4`, `fs-1…6`,
  `align-middle`, `list-unstyled`, `position-*`, `overflow-*`, `border*`,
  `rounded*`, `stretched-link`, and `sr-only`/`visually-hidden`.
- Structural CSS for the JS widgets: `modal*`, `collapse*`, `fade` (behaviour is
  still driven by the retained Bootstrap JS; `theme.css` already styles
  `.modal-content/header/body/footer`).

Load order: grid → `gcds.css` → `gcds-utility` → **`gc-components.css`** →
bootstrap-icons → jquery-ui → **`theme.css`** (theme wins on overlap; it already
styles `.btn` base, `.btn-primary`, `.card`, `.list-group`, `.form-control`).

### JS: kept (deliberately)
The Bootstrap JS bundle is **retained**, because three widgets still depend on
it: the modal in `role/_transfer_confirm.html`, the collapse in
`organization/organization.html`, and the alert (flash) dismiss. Dropping the JS
without migrating these would break them. Migrating those three to GCDS/HTMX is
the remaining task before the bundle (and the modal/collapse shim CSS) can go.

### Docs
`CLAUDE.md` gained a "GC Design System (frontend UI)" section so new pages follow
the conventions: vendored assets only, GCDS chrome/forms/tokens, decorative-icon
rule, and — for layout — **prefer `<gcds-grid>` / `<gcds-grid-col>` /
`<gcds-container>` and `@cdssnc/gcds-utility` over Bootstrap grid markup**, and
GCDS components over the `gc-components.css` shim, so both can eventually be
removed.

## Why hybrid (not a full rip-out)
A full removal of Bootstrap CSS **and** JS in one pass would have required a
large, unverifiable reimplementation of ~25 component/utility class families
that 14–66 templates each depend on, plus replacing the modal/collapse JS. The
hybrid keeps every existing layout working immediately, scopes the shim to
exactly the classes in use, and lets the rest convert to GCDS incrementally.

## Validation performed
- `cargo check` passes; static assets (incl. the new CSS) bundle via `build.rs`.
- `bootstrap-grid.min.css` confirmed to contain grid + flex/display + spacing
  utilities and **no** component classes.
- `gc-components.css` brace-balanced; coverage cross-checked against the audited
  list of non-grid Bootstrap classes used in templates.

## Not verified here (needs a running dev server)
- **Visual fidelity is unverified** — this is the phase most likely to surface
  visual regressions (a shim class with slightly different metrics than
  Bootstrap's). Please do a visual pass across pages, especially: buttons
  (outline/sm), badges/chips, alerts/flash, the role-transfer **modal**, the
  organization detail **collapse**, cards, and tables.
- The unused `static/bootstrap/css/bootstrap.min.css` is left in place (not
  referenced); it can be deleted once the shim is confirmed complete.

## Manual test checklist (when a dev server is up)
- [ ] Every page's layout (rows/columns/spacing) is unchanged.
- [ ] Buttons, badges, alerts, forms, cards, tables look correct in light + dark.
- [ ] Flash messages dismiss; the role-transfer modal opens/closes; the
      organization-detail collapse toggles.
- [ ] `npm run a11y` shows no new violations.
