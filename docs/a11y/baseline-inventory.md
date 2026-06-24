# Phase 0 — Accessibility baseline inventory

**Plan:** [GC Design System & accessibility compliance plan](../GC_DESIGN_SYSTEM_COMPLIANCE.md)
**Date:** 2026-06-21
**Method:** static analysis of `templates/`, `static/css/theme.css`, and
`templates/base.html`. Live automated results (axe/pa11y) are pending a running
dev server — see [`a11y/README.md`](../../a11y/README.md).

This is the "before" snapshot the migration drives down from. No UI was changed
in Phase 0.

## Template corpus

- **103** template files total (`templates/**/*.html`), including HTMX partials,
  macros, and email templates.
- **77** contain an `<h1>`; **17** content templates do not (classified below).

## Findings

### F1 — No skip link (WCAG 2.4.1, blocker)
`base.html` has no "skip to main content" link and the content region is a
`<div class="row">`, not a `<main>` landmark. The only `sr-only` usage is the
"(current)" nav text. **Fix in Phase 1.**

### F2 — No Federal Identity Program elements (FIP, blocker)
No Government of Canada signature, no Canada wordmark, no compliant
header/footer. Header is a Bootstrap `navbar navbar-dark bg-dark` with a custom
"epicenter" brand. **Fix in Phase 1.**

### F3 — Missing mandatory page furniture
No breadcrumbs, no "Date modified", no terms/privacy footer links anywhere in
the corpus. **Fix in Phases 1 & 4.**

### F4 — Heading structure
17 content templates lack an `<h1>`. Triage:

| Template | Type | Action |
|---|---|---|
| `org_chart/node.html`, `org_chart/panel.html`, `org_chart/add_role_form.html`, `org_chart/add_team_form.html`, `org_chart/add_tier_form.html` | HTMX partial (rendered into a parent page) | OK — verify parent provides `<h1>` |
| `role/_matches.html`, `role/_transfer_confirm.html`, `role/_transfer_confirm_inner.html` | HTMX partial | OK — verify parent |
| `org_tier/org_tier_list.html`, `organization/organization_list.html`, `person/person_list.html`, `role/role_list.html`, `skill/skill_list.html`, `team/team_list.html` | HTMX list partial | OK — verify parent |
| `skill/skill_picker.html`, `skill/skill_select.html` | HTMX partial | OK — verify parent |

None appear to be standalone full pages, but the Phase 4 heading audit must
confirm each is only ever served as a fragment and that every full page has
exactly one `<h1>` with no skipped levels (several pages use visual classes like
`display-4`/`h4` that must still map to a logical order).

### F5 — Icons need an accessibility pass
Bootstrap Icons (`bi bi-*`) are used heavily — e.g. `bi-plus-lg` (×22),
`bi-people` (×18), `bi-pencil` (×14), `bi-info-circle` (×13). Only **2**
`aria-hidden` attributes exist in the whole corpus, so most decorative icons are
**not** hidden from assistive tech, and icon-only controls need accessible
names. Audit all `bi bi-*` usages in Phase 4: decorative → `aria-hidden="true"`;
meaningful/standalone → provide a text label.

### F6 — External CDN dependencies (CSP / air-gap)
`base.html` loads Bootstrap Icons and ECharts from `cdn.jsdelivr.net`. For a GC
deployment these must be vendored locally (Phase 1). Fonts will likewise be
served locally (Lato/Noto Sans), not from Google Fonts.

### F7 — Hard-coded colours bypass tokens
Hard-coded hex values appear in `org_chart/builder.html`, `org_chart/node.html`,
`analytics/coverage.html`, and the `viz.html` / `charts.html` macros (domain
chips `#842029`/`#084298`/…; capability pips; `theme.css` code text `#00ff41`).
These must be re-derived from GCDS tokens and AA-contrast-checked in both themes
(Phase 2).

### F8 — Inline styles
12 templates use inline `style="…"` (analytics pages, org-chart builder, several
`*_index` pages, `team.html`, macros). Review during component migration; move
to token-driven classes where practical.

### F9 — Dark theme is the compliant baseline
`theme.css` defaults to dark (`localStorage` default `'dark'`) with low-contrast
accents (notably neon `--code-text: #00ff41`). GC's baseline is light. Make
light the default and either AA-verify every dark token pair or gate dark mode
as an explicit opt-in (Phase 1/2).

### F10 — Form accessibility gaps
- Required fields signalled only by a bare red `*` (`forms.html`); no
  programmatic required pattern, no error summary, no per-field error
  association.
- `index.html` capability filter uses a `<select>` with a `selected disabled`
  "Choose Level" placeholder — a known SR/keyboard antipattern.
- Hard-coded English strings bypass Fluent in `index.html` / `base.html`
  ("Manage your organizational workforce…", "Analytics", "Search Person", etc.)
  — an Official Languages gap as well as an a11y one.
**Fix in Phase 3.**

### Current assistive-tech coverage (baseline)
`aria-*` usage across the corpus: `aria-label` ×12, `aria-expanded` ×4,
`aria-labelledby` ×3, `aria-valuenow/min/max` ×2 each, `aria-hidden` ×2,
`aria-haspopup` ×2, `aria-controls` ×2. Sparse — expected to grow substantially
as GCDS components (which carry correct ARIA) replace hand-rolled markup.

## Live axe/pa11y results

> **Not yet captured** — requires a running dev server (no DB/API in the
> authoring environment). Run `npm run a11y:public` and `npm run a11y` per
> [`a11y/README.md`](../../a11y/README.md) and paste per-route error counts here
> as the numeric baseline.

| Route | axe errors | htmlcs errors | Notes |
|---|---|---|---|
| `/en` | _tbd_ | _tbd_ | |
| `/en/organizations` | _tbd_ | _tbd_ | |
| `/en/person/new` | _tbd_ | _tbd_ | form |
| `/en/analytics/coverage` | _tbd_ | _tbd_ | charts |
| 404 page | _tbd_ | _tbd_ | |
