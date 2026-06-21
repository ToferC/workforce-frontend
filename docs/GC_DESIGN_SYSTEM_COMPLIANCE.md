# Government of Canada Design System & Accessibility Compliance Plan

**Status:** Proposed — ready for implementation
**Audience:** Implementing agent / developer
**Target:** Bring the Workforce frontend into alignment with the
[GC Design System (GCDS)](https://design-system.canada.ca/), the
[Canada.ca content & design specifications](https://design.canada.ca/), and
the federal accessibility obligations (WCAG 2.1 AA / Accessible Canada Act),
while preserving the existing UX (dashboards, org chart builder, analytics,
HTMX progressive enhancement).

---

## 1. Why this matters (the obligations)

Government of Canada web applications are bound by a stack of mandatory rules.
The implementing agent should treat these as acceptance criteria, not
suggestions:

| Obligation | Source | What it requires |
|---|---|---|
| **WCAG 2.1 Level AA** | [Standard on Web Accessibility](https://www.tbs-sct.canada.ca/pol/doc-eng.aspx?id=23601) + Accessible Canada Act | All content perceivable, operable, understandable, robust. AA conformance for every page and interactive component. |
| **Federal Identity Program (FIP)** | [TBS FIP policy](https://www.canada.ca/en/treasury-board-secretariat/services/government-communications/federal-identity-program.html) | The Government of Canada signature (flag + wordmark) top-left, and the "Canada" wordmark bottom-right, on every page. |
| **Official Languages** | Official Languages Act | Full English/French parity, a language toggle on every page, correct `lang`/`hreflang` attributes. The app already uses Fluent i18n — build on it. |
| **Canada.ca design specifications** | [design.canada.ca](https://design.canada.ca/) | Mandatory page elements: signature, language toggle, search (optional for internal apps), breadcrumbs, "Date modified", terms/privacy footer, Canada wordmark. |
| **GC Design System** | [design-system.canada.ca](https://design-system.canada.ca/) | Use GCDS components, tokens, and patterns so the app looks and behaves like a GC service. |

> **Internal-app note:** This is an internal workforce-management tool, not a
> public Canada.ca page. The Canada.ca *theme* (GCWeb/WET) is intended for
> public content; the **GC Design System (GCDS) component library is the
> correct choice for an application** like this. Some Canada.ca-specific
> elements (e.g. global search of canada.ca, the full institutional menu) are
> not appropriate. This plan uses GCDS and applies the FIP + accessibility
> rules that apply to *all* GC web properties.

---

## 2. What the GC Design System actually gives us

GCDS ships three independently-usable packages (all MIT/OGL, published on npm
by the Canadian Digital Service under `@cdssnc/`):

1. **`@cdssnc/gcds-components`** — framework-agnostic Web Components (built with
   Stencil). They work in any HTML page via a script tag + custom elements;
   no React/Vue/build step required. This is the key fit for a Tera-rendered
   server-side app. Examples: `<gcds-header>`, `<gcds-footer>`, `<gcds-button>`,
   `<gcds-input>`, `<gcds-select>`, `<gcds-textarea>`, `<gcds-checkbox>`,
   `<gcds-radio-group>`, `<gcds-fieldset>`, `<gcds-date-input>`,
   `<gcds-error-message>`, `<gcds-error-summary>`, `<gcds-notice>`,
   `<gcds-breadcrumbs>`, `<gcds-pagination>`, `<gcds-card>`, `<gcds-grid>`,
   `<gcds-container>`, `<gcds-heading>`, `<gcds-text>`, `<gcds-link>`,
   `<gcds-date-modified>`, `<gcds-lang-toggle>`, `<gcds-search>`,
   `<gcds-top-nav>` / `<gcds-side-nav>` / `<gcds-nav-link>` / `<gcds-nav-group>`,
   `<gcds-signature>`, `<gcds-icon>`, `<gcds-details>`, `<gcds-stepper>`.
2. **`@cdssnc/gcds-tokens`** — design tokens (colour, typography, spacing,
   layout) as CSS custom properties / SCSS. The single source of truth for
   GC-compliant colour and spacing.
3. **`@cdssnc/gcds-utility`** — utility CSS classes (spacing, layout, display)
   for cases where a component isn't enough.

Key GCDS conventions to honour:

- **Typography:** GCDS uses **Lato** for headings/body with **Noto Sans** as
  the secondary/fallback face. Base font size is larger than Bootstrap's
  default (≈20px body) for readability. Tokens expose the full type scale.
- **Colour:** Primary GC blue (`#26374A` deep blue for header/structure),
  link blue (`#284162` / hover `#0535D2`), FIP red (`#af3c43`) reserved for the
  signature, and a constrained neutral palette — all AA-contrast-checked. Do
  **not** invent colours; consume tokens.
- **Spacing:** an 8px-based spacing scale via tokens (`--gcds-spacing-*`).
- **Light theme is the GC default.** The current app defaults to a dark theme
  with neon-green code text — that is not GC-compliant as a default (see §5).

GCDS components are themselves built to WCAG 2.1 AA, so adopting them removes a
large class of accessibility bugs (focus states, labels, contrast, keyboard
behaviour) for free.

---

## 3. Current-state assessment (this repo)

Observed in the codebase (`templates/base.html`, `static/css/theme.css`,
`templates/macros/forms.html`, `templates/index.html`, error pages):

**Strengths to keep**
- Bilingual already wired through Fluent (`fluent(key=…, lang=lang)`), with
  `i18n/en` + `i18n/fr` and a working language toggle. Strong foundation for
  the OL requirement.
- Server-rendered Tera templates with a single `base.html` layout → one place
  to inject the GC header/footer/skip-link.
- A `forms.html` macro layer → swapping field rendering is centralized.
- HTMX progressive enhancement with documented "plain POST still works" rule.
- `*:focus-visible` outline and some `aria-*`/`sr-only` usage already present.
- CSS custom properties already used for theming → easy to repoint at GCDS
  tokens.

**Gaps / non-compliance**
1. **No FIP elements.** No Government of Canada signature, no Canada wordmark,
   no compliant global header/footer. The header is a generic Bootstrap
   `navbar navbar-dark bg-dark` with a custom brand string ("epicenter").
2. **No skip-to-main-content link** — a WCAG 2.4.1 (bypass blocks) failure.
3. **No breadcrumbs**, **no "Date modified"**, **no terms/privacy footer** —
   mandatory Canada.ca page elements are absent.
4. **Dark-mode-as-default** with `--code-text: #00ff41` on near-black — likely
   AA contrast and brand-tone problems; not a GC look. (Dark mode itself is
   fine as an *opt-in* preference, but must not be the compliant baseline and
   every token pair must pass AA.)
5. **Bootstrap 4.5.3 + jQuery + jQuery UI** drive the visual language. These
   are not GC-styled and add weight; jQuery UI widgets (datepicker, etc.) are
   not accessible to GC standard.
6. **Typography** is the system sans stack, not Lato/Noto Sans.
7. **Forms** use Bootstrap markup with a bare red `*` for "required" and no
   programmatic error association, no error summary pattern, and `<select>`
   with `selected disabled` placeholder option (a known SR/keyboard pitfall).
8. **Icons** come from Bootstrap Icons via CDN — decorative icons need
   `aria-hidden`; meaningful ones need text alternatives. Mixed usage today.
9. **External CDNs** in `base.html` (Bootstrap Icons, ECharts) — offline/air-gap
   and CSP concerns for a GC deployment; vendor locally.
10. **Headings**: some pages use `display-4`/visual sizing that may not match a
    logical heading order; needs an audit for one `<h1>` per page and no skips.
11. **`lang` is set on `<html>`** (good) but interactive controls (theme toggle,
    icon-only buttons) and the language toggle need `lang`/`hreflang`/`aria`
    review.

---

## 4. Implementation strategy

Two viable paths. **Recommendation: Path A (adopt GCDS web components)** — it is
the lowest-risk way to actually *be* compliant rather than approximate it, and
the components are framework-agnostic so they drop into Tera output.

### Path A — Adopt GCDS web components (recommended)
Layer GCDS over the existing server-rendered pages incrementally:
- Vendor `@cdssnc/gcds-components`, `gcds-tokens`, and `gcds-utility` into
  `static/` (no CDN, for CSP/air-gap), load in `base.html`.
- Replace the global chrome first (header, footer, skip link, signature,
  wordmark, language toggle) — highest compliance payoff, lowest churn.
- Migrate forms and structural components (cards, grid, breadcrumbs, notices)
  page-by-page behind the existing Fluent labels.
- Repoint `theme.css` custom properties at GCDS tokens; retire Bootstrap
  page-by-page until it can be removed.
- Keep HTMX. GCDS components emit standard DOM events and standard form
  controls, so `name`/`value` submission and "plain POST still works" hold.

**Risk:** GCDS components are not styled by Bootstrap; during migration two
visual languages coexist. Mitigate by migrating the global chrome + tokens
first so the palette/typography are unified even before every page is done.

### Path B — Hand-roll GC styling on top of Bootstrap
Recreate GC look using tokens + custom CSS, keep Bootstrap markup. Faster to a
"looks GC-ish" state but you re-implement (and must re-test) accessibility that
GCDS already guarantees, and you will drift from the system over time. Only
choose this if a hard constraint forbids web components.

The phases below assume **Path A**.

---

## 5. Phased work plan

### Phase 0 — Foundations & audit (no visual change)
- [ ] Add `docs/` note + an automated a11y baseline: wire **axe-core** (or
      `pa11y`) against the running dev server for the key routes (`/`, an index
      list, a detail page, a create form, an analytics page, the org-chart
      builder, each error page). Capture the current violation count as the
      baseline to drive down.
- [ ] Add an **HTML validation** + heading-order check to the same harness.
- [ ] Inventory every template's heading structure, colour usage, and icon
      usage into a checklist (one row per template).
- [ ] Decide token integration: import `gcds-tokens` CSS and map the existing
      `--bg-*`, `--text-*`, `--color-*`, `--border-*`, spacing, and radius
      variables in `theme.css` onto GCDS token values.

**Acceptance:** baseline a11y report committed under `docs/a11y/`; no UI change.

### Phase 1 — Vendor GCDS & global chrome (highest payoff)
- [ ] Vendor GCDS assets into `static/gcds/` (components loader, CSS, tokens,
      utility, fonts — Lato + Noto Sans served locally, not from Google Fonts).
      Update `build.rs` static bundling and remove the Bootstrap-Icons/ECharts
      **CDN** `<link>/<script>` in favour of local copies (CSP-friendly).
- [ ] In `base.html`, immediately after `<body>`, add a **skip link** to
      `#main-content` (WCAG 2.4.1) and mark the content `<div>` as
      `<main id="main-content" tabindex="-1">`.
- [ ] Replace the Bootstrap navbar with **`<gcds-header>`** carrying:
      - the **Government of Canada signature** (`<gcds-signature variant="signature">`),
      - the **language toggle** (`<gcds-lang-toggle>` or the header's
        `lang-href`) wired to the existing `/toggle_language{{ path }}` route,
      - the app's primary navigation (the current "Explore"/Analytics menus)
        rendered via `<gcds-top-nav>`/`<gcds-nav-group>`/`<gcds-nav-link>` with
        Fluent labels, plus the authenticated user menu and login/register.
- [ ] Replace the footer with **`<gcds-footer>`** including the **Canada
      wordmark** and the required links (terms & conditions, privacy). Keep the
      existing attribution/licence text in the footer's contextual band.
- [ ] Add **`<gcds-date-modified>`** to the footer/base layout.
- [ ] Keep the theme toggle **only if** every GCDS-token light/dark pair passes
      AA contrast; otherwise make **light the default** and gate dark mode as an
      explicit, AA-verified opt-in. Replace the emoji-only toggle button with an
      accessible, labelled control (it already has `aria-label`; ensure name +
      role + state are exposed and it is reachable/operable by keyboard).

**Acceptance:** every page shows the GC signature (top-left) and Canada wordmark
(bottom-right), a working skip link, a language toggle, and a date-modified;
header/footer pass axe with zero new violations.

### Phase 2 — Tokens, typography & colour
- [ ] Point `theme.css` variables at `gcds-tokens`; remove ad-hoc colours
      (`#00ff41` code text, hard-coded domain-chip hexes) in favour of
      token-derived, AA-checked values. The `viz.html`/`charts.html` macro
      colours (domain chips, capability pips, status chips) must be re-derived
      from tokens and contrast-tested in **both** themes.
- [ ] Adopt **Lato/Noto Sans** and the GCDS type scale; align `line-height`
      and base size to GCDS (larger, readable defaults).
- [ ] Ensure ECharts visualizations read colours from CSS variables so charts
      stay on-palette and AA-legible after the `themechange` event (already
      dispatched in `base.html`). Add accessible fallbacks: every chart needs a
      text/table equivalent or `aria-label` + description (charts are images to
      a screen reader).

**Acceptance:** all text/UI passes AA contrast in the default theme (and in dark
mode if retained); fonts are GC fonts served locally.

### Phase 3 — Forms (largest accessibility surface)
Rework `templates/macros/forms.html` to GCDS form components/patterns:
- [ ] Replace `text_input`/`textarea`/`select`/`date_input`/`checkbox` macros
      with `<gcds-input>`, `<gcds-textarea>`, `<gcds-select>`,
      `<gcds-date-input>`, `<gcds-checkbox>` — keeping the same macro signatures
      and Fluent labels so callers don't change.
- [ ] Required fields: use the GCDS required pattern (don't rely on a lone red
      `*`); mark optional fields per GC guidance instead, and ensure the
      required state is programmatically determinable.
- [ ] Server-side validation errors must render with **`<gcds-error-message>`**
      tied to each field (`aria-describedby`) **and** a top-of-form
      **`<gcds-error-summary>`** that links to the offending fields and receives
      focus on submit (WCAG 3.3.1/3.3.3). Wire this through the existing
      flash/`generate_basic_context` mechanism and the form re-render path.
- [ ] Fix the placeholder-as-label `<select>` antipattern in `index.html`
      ("Choose Level" `selected disabled`) — use a proper label + first
      neutral option.
- [ ] Preserve CSRF: keep the hidden `csrf_token` field (`forms::csrf`) and
      `security::verify_csrf_token`; GCDS form fields still submit `name=value`.
- [ ] Confirm HTMX flows: GCDS controls inside HTMX-swapped partials (e.g. the
      org-chart inline add-role form) must re-bind and the
      `htmx:afterSettle` military/civilian sync script must still find fields by
      `name`.

**Acceptance:** every create/edit form is keyboard-only completable, errors are
announced and linked, and labels/help/required state pass axe.

### Phase 4 — Structural components & navigation
- [ ] Add **`<gcds-breadcrumbs>`** to detail/index/form pages (e.g. Home ›
      Organizations › {name}). Drive trail from route context.
- [ ] Replace Bootstrap `card`/`list-group`/grid in index and detail templates
      with `<gcds-card>`, `<gcds-grid>`, `<gcds-container>` where it does not
      regress the bespoke `detail-layout`/org-chart UIs. Keep custom layouts
      (org chart builder, analytics dashboards) but restyle to tokens.
- [ ] Replace flash `alert` markup with **`<gcds-notice>`** (success/danger/
      info/warning → GCDS notice types) while keeping the dismiss affordance
      accessible.
- [ ] Replace any pagination with **`<gcds-pagination>`**.
- [ ] Audit all icon usage: decorative Bootstrap-Icons get `aria-hidden="true"`;
      icon-only controls get an accessible name. Prefer `<gcds-icon>` where it
      maps.
- [ ] Error pages (`404`, `not_authorized`, `not_found`,
      `internal_server_error`) get the same chrome, a real `<h1>`, and
      GC-styled content.

**Acceptance:** consistent GC navigation/structure across all routes; logical
single-`<h1>` heading order per page.

### Phase 5 — Retire Bootstrap / jQuery UI & finalize
- [ ] Remove Bootstrap CSS/JS and jQuery UI once no template depends on them;
      replace any remaining jQuery-UI widgets (datepickers, sliders, sortables)
      with GCDS components or accessible native equivalents (`<input type=
      "date">`, `<input type="range">`, the capability slider). Keep jQuery only
      if still required by a specific interaction, otherwise drop it.
- [ ] Re-run the full axe/pa11y suite; drive new violations to zero and
      document any deferred items.
- [ ] Update `CLAUDE.md` "GraphQL Client Conventions"/template guidance to point
      at GCDS macros instead of Bootstrap, and note the FIP/skip-link/date-
      modified requirements for new pages.

**Acceptance:** Bootstrap/jQuery-UI removed (or explicitly justified); a11y
suite green; docs updated.

---

## 6. Cross-cutting accessibility checklist (apply to every page)

- [ ] **1.1.1** Non-text content: alt text / `aria-hidden` for icons; text or
      table alternative for every ECharts visualization.
- [ ] **1.3.1** Info & relationships: semantic landmarks (`<header> <nav>
      <main> <footer>`), labelled form controls, real table headers (`<th
      scope>`).
- [ ] **1.4.3 / 1.4.11** Contrast: text ≥ 4.5:1, UI/graphics ≥ 3:1 — verify in
      light **and** dark themes.
- [ ] **2.1.1 / 2.1.2** Keyboard operable, no traps (dropdowns, modals, org
      chart builder, HTMX interactions).
- [ ] **2.4.1** Skip link to main content.
- [ ] **2.4.2** Unique, descriptive `<title>` per page (already per-template;
      verify French parity).
- [ ] **2.4.3** Logical focus order; **2.4.7** visible focus (keep/standardize
      the existing `focus-visible` outline via tokens).
- [ ] **2.4.6** Descriptive headings & labels; one `<h1>` per page.
- [ ] **3.1.1 / 3.1.2** `lang` on `<html>` and on any inline
      passages in the other official language.
- [ ] **3.2.x** Predictable: language toggle and theme toggle don't change
      context unexpectedly.
- [ ] **3.3.1 / 3.3.3** Error identification + suggestions via error summary +
      per-field messages.
- [ ] **4.1.2 / 4.1.3** Name/role/value for all custom controls; status
      messages (flash) announced via `role="status"`/`aria-live` or
      `<gcds-notice>`.
- [ ] `prefers-reduced-motion`: gate the global `transition`/`transform` and
      `scroll-behavior: smooth` in `theme.css` behind a reduced-motion query.

---

## 7. Bilingual (Official Languages) checklist

- [ ] Every new GCDS component label sourced from Fluent (`i18n/en`, `i18n/fr`)
      — no hard-coded English (note current `index.html`/`base.html` strings
      like "Manage your organizational workforce…", "Analytics", "Dashboard",
      "Search Person" that bypass Fluent — migrate them).
- [ ] Language toggle present on every page (via `<gcds-header>`), preserving
      the current path (`/toggle_language{{ path }}`).
- [ ] `hreflang` on the language toggle link; `lang` correct on `<html>`.
- [ ] French content reaches full parity for all new strings and error
      messages.

---

## 8. Key files the implementing agent will touch

| Area | Files |
|---|---|
| Global layout, chrome, skip link, scripts | `templates/base.html` |
| Tokens, colours, typography, dark mode, motion | `static/css/theme.css` |
| Vendored assets + build bundling | `static/`, `build.rs` |
| Forms | `templates/macros/forms.html` + every `*_form.html` |
| Viz/badges/charts colour | `templates/macros/viz.html`, `templates/macros/charts.html` |
| Landing & search | `templates/index.html` |
| Breadcrumbs/cards/notices | index + detail templates across `person/`, `organization/`, `role/`, `team/`, `task/`, `work/`, `product/`, `publication/`, `skill/`, `capability/`, `org_tier/`, `org_chart/`, `analytics/`, `users/` |
| Errors | `templates/errors/*.html` |
| i18n | `i18n/en`, `i18n/fr` |
| Docs/conventions | `CLAUDE.md`, `docs/a11y/` (new) |

---

## 9. Definition of done

1. Every page renders the GC signature (top-left) and Canada wordmark
   (bottom-right), a skip link, a language toggle, breadcrumbs (where
   applicable), and a date-modified.
2. Default theme is GC-compliant (light, Lato/Noto Sans, token colours);
   optional dark mode passes AA on every token pair or is removed.
3. All forms use GCDS controls with linked per-field errors + an error summary,
   keyboard-completable, CSRF intact, HTMX flows working.
4. axe-core/pa11y reports **zero** AA violations on the audited routes; manual
   keyboard + screen-reader smoke test of the core flows passes.
5. Full EN/FR parity; no hard-coded user-facing strings.
6. External CDNs removed; GCDS + fonts vendored locally for CSP/air-gap.
7. Bootstrap/jQuery-UI removed or their continued use explicitly justified.
8. `CLAUDE.md` updated so new pages inherit these rules by default.

---

## 10. Suggested sequencing for incremental PRs

1. **PR 1 — Audit harness + baseline** (Phase 0).
2. **PR 2 — Vendor GCDS + tokens + global header/footer/skip-link/wordmark**
   (Phases 1–2). *Biggest compliance jump.*
3. **PR 3 — Forms macros + error summary pattern** (Phase 3).
4. **PR 4 — Breadcrumbs, cards, notices, error pages** (Phase 4).
5. **PR 5 — Retire Bootstrap/jQuery-UI, finalize a11y, update docs** (Phase 5).

Each PR keeps the app shippable and "plain POST still works"; the a11y harness
gates each step so violations only ever go down.
