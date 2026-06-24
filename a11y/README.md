# Accessibility audit harness

Phase 0 of the [GC Design System & accessibility compliance plan](../docs/GC_DESIGN_SYSTEM_COMPLIANCE.md).

This harness runs automated WCAG 2.1 AA checks (axe-core + HTML CodeSniffer)
against the running app so each migration PR can be gated on "violations only
go down". Automated tools catch ~30–40% of WCAG issues — they are a floor, not
a substitute for the manual keyboard + screen-reader testing called out in the
plan.

## Prerequisites

- Node.js 18+ and npm (verified available: Node 22, npm 10).
- A running dev server at `http://127.0.0.1:8088` (`cargo run`) with a database
  and seeded data. The authenticated routes need an admin account — reuse the
  `ADMIN_EMAIL` / `ADMIN_PASSWORD` from the app's `.env`.

## Install

```bash
cd a11y
npm install
```

## Run

Public, no-auth pages (works without seeded data):

```bash
npm run a11y:public
```

Full authenticated crawl (lists, detail, forms, analytics):

```bash
ADMIN_EMAIL="you@example.com" ADMIN_PASSWORD="secret" npm run a11y
```

Override the base URL with `A11Y_BASE_URL` if the server runs elsewhere.

## Files

| File | Purpose |
|---|---|
| `.pa11yci.public.json` | Unauthenticated routes (`/en`, `/fr`, `/en/about`, login, register, a 404). |
| `pa11yci.auth.js` | Authenticated routes; logs in via env-supplied admin creds before each page. |
| `package.json` | `npm run a11y` / `npm run a11y:public`. |

## Capturing the baseline

The container these docs were authored in cannot run the Rust server (no
database/API), so the **live violation baseline has not yet been captured**.
The first task for whoever has a running dev server:

1. Run both commands above.
2. Save the console output to `docs/a11y/baseline-report.txt`.
3. Record the total error count per route in
   [`docs/a11y/baseline-inventory.md`](../docs/a11y/baseline-inventory.md)
   (the "Live axe/pa11y results" section).

That number is the figure every subsequent phase must drive toward zero.

## CI (later)

Once the baseline is green-ish, wire `npm run a11y:public` into CI against an
ephemeral server instance so regressions are caught on PRs. The authenticated
crawl needs seeded data + secrets, so it may stay a manual/nightly job.
