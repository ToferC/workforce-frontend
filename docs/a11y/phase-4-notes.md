# Phase 4 — Structural components & navigation (implementation notes)

**Plan:** [GC Design System & accessibility compliance plan](../GC_DESIGN_SYSTEM_COMPLIANCE.md)
**Scope:** formatting / accessibility of structure & navigation. No business
logic, data, or HTMX changed.

## What landed

### Icon accessibility (WCAG 1.1.1)
- Added `aria-hidden="true"` to **291 decorative Bootstrap icons across 85
  templates** (scripted; idempotent). Decorative `<i class="bi …">` glyphs are
  now hidden from assistive tech, so screen readers read the adjacent text
  label instead of an icon name.
- The only **4 icon-only controls** (in `admin/users.html`: edit / grant access
  / enable / disable) now carry an `aria-label` (from existing Fluent keys), so
  they have an accessible name now that their icon is hidden.

### Breadcrumbs (GCDS navigation)
- Added a `{% block breadcrumb %}` to `base.html`, rendered at the top of
  `<main>` (empty by default).
- The **10 entity create/edit forms** (organization, person, role, team, task,
  work, product, publication, skill, org_tier) now render a
  `<gcds-breadcrumbs hide-canada-link>` trail of **Home › {Section}**
  (`hide-canada-link` because this is an internal app, not canada.ca). The
  current page is intentionally not a crumb, per the GC breadcrumb pattern.
- Added a `home` Fluent key (EN: "Home" / FR: "Accueil").

### Flash messages
- Fixed the dismiss button: it used Bootstrap 5 markup (`btn-close` /
  `data-bs-dismiss`) while the vendored Bootstrap is 4.5.3, so it was
  non-functional. Replaced with the BS4-compatible
  `<button class="close" data-dismiss="alert">` + `aria-hidden` glyph. The
  alert already exposes `role="alert"` for announcement.

### Error pages
- Reviewed all four (`404`, `not_found`, `not_authorized`,
  `internal_server_error`). They already extend `base.html` (so they inherit the
  Phase 1 GC chrome), already have a real `<h1>`, and use Fluent — no change
  needed beyond the icon `aria-hidden` pass above.

## Deliberately deferred (with rationale)
- **`<gcds-card>` / `<gcds-grid>` migration.** The existing Bootstrap cards,
  list-groups, and grid were already repointed at GCDS tokens in Phase 2, so
  they're on-palette. Swapping the markup wholesale is high blast-radius across
  many templates (and risks regressing the bespoke `detail-layout` and org-chart
  UIs) for low marginal value. Folded into the Phase 5 Bootstrap-retirement
  work instead.
- **`<gcds-notice>` for flash messages.** `gcds-notice` requires a title and is
  a page-level banner, which is too heavy for transient per-action flash. Kept
  the (now-fixed, `role="alert"`) Bootstrap alert.
- **`<gcds-pagination>`.** No pagination exists in the templates today — nothing
  to convert.
- **Breadcrumbs on index/detail pages.** Index pages would show only "Home";
  detail-page trails (Home › Section › Item) are valuable but spread across many
  templates. Deferred to keep this phase bounded and consistent (all *forms*
  have breadcrumbs); add detail-page trails in a follow-up.

## Validation performed
- `gcds-breadcrumbs` / `gcds-breadcrumbs-item` confirmed as real custom-element
  tags in the vendored loader.
- Tera balance checked: breadcrumb `block`/`endblock` pairs match in all 11
  touched files; `base.html` blocks balanced.
- All 291 icons verified to carry `aria-hidden`; 4 icon-only controls labeled.

## Not verified here (needs a running dev server)
- Visual rendering of breadcrumbs, the fixed flash dismiss, and the icon pass
  are not runtime-tested (no DB/API in this environment). Manual + `a11y/`
  harness check still outstanding from earlier phases.

## Manual check when a dev server is up
- [ ] Entity create/edit forms show a Home › Section breadcrumb that navigates.
- [ ] Screen reader skips decorative icons; the admin user-row action buttons
      announce their labels.
- [ ] Flash messages can be dismissed.
- [ ] `npm run a11y` shows no new violations.
