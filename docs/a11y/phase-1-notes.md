# Phase 1 — GCDS global chrome (implementation notes)

**Plan:** [GC Design System & accessibility compliance plan](../GC_DESIGN_SYSTEM_COMPLIANCE.md)
**Scope:** vendor GCDS, replace the global header/footer, add the FIP signature +
Canada wordmark, skip link, language toggle, and date-modified. No per-page
content migration yet (that's Phases 3–4).

## What landed

### Vendored assets (no CDN — CSP / air-gap ready)
All served from `/static`, bundled at build time by `build.rs`:

- `static/gcds/` — `@gcds-core/components@1.3.1` loader (`gcds.esm.js` + entry
  chunks), `gcds.css` (design tokens + component styles), and
  `gcds-utility.min.css` (`@cdssnc/gcds-utility@1.11.0`).
- `static/gcds/fonts/` — Lato, Noto Sans, and Noto Sans Mono
  (`@gcds-core/fonts`), with local `@font-face` CSS. The Google-Fonts `@import`
  and the remote `gcds-icons` URLs were stripped from `gcds.css` and repointed
  at the vendored files.
- `static/bootstrap-icons/` — Bootstrap Icons 1.11.3 (was a jsDelivr CDN link).
- `static/echarts/echarts.min.js` — ECharts 5.5.1 (was a jsDelivr CDN script).

`base.html` no longer references any external host.

### `templates/base.html`
- **`<gcds-header>`** with `skip-to-href="#main-content"` (renders the skip
  link — WCAG 2.4.1) and `lang-href="/toggle_language{{ path }}"` (the existing
  bilingual toggle route → official-languages requirement). The header renders
  the **Government of Canada signature** (FIP).
- Primary nav rebuilt as **`<gcds-top-nav>`** with the app name in the `home`
  slot and `<gcds-nav-group>`s for Explore, Analytics, and the signed-in user
  menu; `<gcds-nav-link>`s for login/register when signed out. All labels come
  from Fluent (except the Analytics sub-items — see limitations).
- Content wrapped in **`<main id="main-content" tabindex="-1">`** (skip-link
  target + main landmark).
- **`<gcds-footer display="compact">`** renders the mandatory **Canada
  wordmark** plus contextual links (source code, about). The existing
  attribution/licence prose is preserved in a band above it.
- **`<gcds-date-modified>`** added to the footer region.
- Theme toggle retained but **defaults to light** (GC baseline); dark mode is an
  opt-in. The toggle script no longer manipulates the (removed) Bootstrap
  navbar, and its CSS now reads from theme variables so it's visible on the
  light page.

Verified: `cargo check` passes (static assets bundle cleanly; only pre-existing
warnings).

## Known limitations / follow-ups

1. **`<gcds-nav-link>` requires JavaScript.** The clickable `<a>` is created in
   the component's shadow DOM, so with JS disabled the nav renders as text only.
   Acceptable for this internal, JS-required app, but note it's a regression
   from plain `<a>` links. (Forms remain plain POST + redirect per the repo
   convention — unaffected.)
2. **Analytics sub-menu labels are hard-coded English** ("Capability Coverage",
   etc.) — carried over verbatim from the old navbar, which had no Fluent keys
   for them. Add `analytics-coverage` / `analytics-delivery` / … keys to
   `i18n/{en,fr}` and swap them in (Official Languages clean-up, Phase 4/7).
3. **`<gcds-date-modified>` date is static** (`2026-06-21`). Wire it to a real
   "last updated" value from handler context where meaningful.
4. **Bootstrap + GCDS coexist.** GCDS components are shadow-DOM isolated, so the
   chrome is GC-styled while page bodies remain Bootstrap until Phases 2–5.
   `theme.css` still overrides body typography; Phase 2 repoints it at GCDS
   tokens/fonts.
5. **Live axe/pa11y baseline still pending** a running dev server (no DB/API in
   this environment). Run the `a11y/` harness against the new chrome and record
   numbers in `baseline-inventory.md` — this is the first verification task once
   a server is available.

## Manual test checklist (run when a dev server is up)
- [ ] GC signature shows top-left; Canada wordmark bottom-right on every page.
- [ ] Skip link appears on first Tab and moves focus to `#main-content`.
- [ ] Language toggle switches EN/FR and preserves the path.
- [ ] Nav groups open/close by keyboard; focus order is logical.
- [ ] Theme toggle defaults to light, persists choice, and is visible/operable.
- [ ] `npm run a11y:public` and `npm run a11y` — record violation counts.
