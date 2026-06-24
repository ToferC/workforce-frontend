# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Workforce** is a Rust web application for organizational management in epidemiology/public health contexts. It manages people, roles, capabilities, teams, organizations, and related data within public health workflows.

## Technology Stack

- **Rust** (Edition 2021) with **Actix-Web 3.3.3** web framework
- **Diesel ORM** with **PostgreSQL** database
- **Tera** templating engine for HTML rendering
- **GraphQL** API integration with client queries
- **Fluent Templates** for bilingual support (English/French)
- **Bootstrap** + **jQuery** for frontend styling and interactions
- **SendGrid** for email services

## Development Commands

### Setup
```bash
# Create .env file with required environment variables (see README.md)
diesel migration run    # Setup database schema
cargo run              # Start development server (http://127.0.0.1:8088)
```

### Development
```bash
cargo build            # Build the application
cargo check            # Quick syntax/type checking
cargo clippy           # Linting (if available)
cargo test             # Run tests (if any exist)
cargo run              # Run development server
```

### Database
```bash
diesel migration run            # Apply all pending migrations
diesel migration generate NAME  # Create new migration
diesel print-schema            # Print current database schema
```

## Architecture

### Core Structure
- **`src/handlers/`** - HTTP request handlers for each entity (person, role, organization, etc.)
- **`src/models/`** - Diesel ORM data models
- **`src/graphql/`** - GraphQL resolvers and type definitions
- **`templates/`** - Tera HTML templates organized by feature
- **`static/`** - CSS, JavaScript, and static assets
- **`migrations/`** - Database schema migrations
- **`queries/`** - GraphQL query definitions
- **`i18n/`** - Internationalization files for EN/FR support

### Key Entry Points
- **`src/main.rs`** - Application bootstrap and server startup
- **`src/lib.rs`** - Library definitions and app-wide utilities
- **`src/handlers/routes.rs`** - HTTP route configuration
- **`schema.graphql`** - Complete GraphQL API schema

### Domain Model
The application centers around organizational management with entities:
- **Person** (individuals with capabilities)
- **Organization** and **OrgTier** (hierarchical structures)
- **Role** and **Team** (positions and groups)
- **Capability** and **Skill** (competencies)
- **Work** and **Task** (assignments and projects)
- **Publication** (research outputs)

### Authentication & Sessions
- Uses **Actix-Identity** for session management
- Email verification workflow via SendGrid
- Role-based access control throughout the application

## Environment Setup

Required `.env` variables:
- `COOKIE_SECRET_KEY` (minimum 32 characters)
- `DATABASE_URL` (PostgreSQL connection string)
- `SENDGRID_API_KEY`
- `ADMIN_NAME`, `ADMIN_EMAIL`, `ADMIN_PASSWORD`
- `ENVIRONMENT=test`

## Code Patterns

- **Handler Pattern**: Each entity has dedicated handlers in `src/handlers/`
- **Template Organization**: Templates mirror the handler structure
- **GraphQL Integration**: Queries defined in `queries/` directory
- **Bilingual Support**: All user-facing strings use Fluent i18n system
- **Static File Compilation**: Build-time asset bundling via `build.rs`

## GraphQL Client Conventions

- `schema.graphql` is the source of truth for generated client types. It must
  stay in sync with `schema.graphqls` in the workforce_analytics repo (the API).
- All API calls go through `post_graphql` in `src/graphql/client.rs`. It sends
  the JWT as `Authorization: Bearer <token>` (the header the API validates) and
  returns `ApiError` instead of panicking when the response carries GraphQL
  errors. Do not hand-roll reqwest calls in entity modules.
- Handlers that render forms or call guarded mutations must enforce access with
  `security::require_role(&session, &lang, MinimumRole::...)`, mirroring the
  API's `user < analyst < operator < admin` hierarchy. Template role checks are
  for hiding buttons only.
- Mutating POST handlers must validate the form's `csrf_token` field with
  `security::verify_csrf_token`. `generate_basic_context` injects `csrf_token`
  and `flash_messages` into every template context; queue user feedback with
  `security::add_flash(session, "success" | "danger", message)`.
- Entity create/edit forms use the macros in `templates/macros/forms.html`
  (`{% import "macros/forms.html" as forms %}`).
- HTMX is vendored at `static/htmx/htmx.min.js` and loaded in `base.html` for
  progressive enhancement; forms must still work as plain POST + redirect.
- Visual vocabulary macros are in `templates/macros/viz.html`
  (`{% import "macros/viz.html" as viz %}`). Use `viz::effort_meter`,
  `viz::status_chip`, `viz::domain_chip`, `viz::capability_scale`, and
  `viz::level_chip` instead of hand-rolled Bootstrap badges for these values.
- **ECharts** is vendored at `static/echarts/echarts.min.js` and loaded in
  `base.html` (no CDN). Chart macros are in
  `templates/macros/charts.html` (`{% import "macros/charts.html" as charts %}`).
  Handlers aggregate JSON and inject it into the context; templates render it via
  `{{ charts::chart(id="...", height="300px") }}` paired with a
  `<script type="application/json" id="...-data">{{ my_json }}</script>` payload.
  Charts do NOT call GraphQL client-side. The `themechange` custom DOM event is
  dispatched on theme toggle so charts re-render correctly.

## GC Design System (frontend UI)

The frontend is being aligned with the
[GC Design System (GCDS)](https://design-system.canada.ca/) and federal
accessibility / Federal Identity Program requirements. The migration plan and
per-phase notes live in `docs/GC_DESIGN_SYSTEM_COMPLIANCE.md` and `docs/a11y/`.
When building or changing frontend pages:

- **Vendored, no CDNs.** GCDS components/tokens/utility, fonts (Lato / Noto
  Sans / Noto Sans Mono), Bootstrap Icons, and ECharts are all served from
  `static/`. Do not reintroduce external `<link>`/`<script>` hosts (CSP /
  air-gap).
- **Global chrome** is GCDS: `base.html` renders `<gcds-header>` (Government of
  Canada signature, skip-to-content link, `/toggle_language` language toggle)
  and `<gcds-footer>` (Canada wordmark) + `<gcds-date-modified>`. Content lives
  in `<main id="main-content">`. Add a breadcrumb trail via the
  `{% block breadcrumb %}` (see the entity `*_form.html` for the
  `<gcds-breadcrumbs hide-canada-link>` pattern).
- **Forms** use the GCDS-backed macros in `templates/macros/forms.html`
  (`gcds-input` / `gcds-textarea` / `gcds-select`, all form-associated so plain
  POST + HTMX submission is unchanged). Put `{{ forms::error_summary() }}` at the
  top of a form and pass server errors via each macro's `error` param. Native
  date input and single checkbox are intentionally kept.
- **Theme/tokens.** `static/css/theme.css` maps the app's CSS custom properties
  onto GCDS tokens; light is the default theme, dark is an opt-in. Use the
  `--color-*` / `--bg-*` / `--text-*` / `--spacing-*` variables, not raw hexes.
- **Icons** are decorative: give every `<i class="bi …">` `aria-hidden="true"`,
  and give icon-only controls an `aria-label`.
- **Grid / layout (hybrid).** Bootstrap is reduced to `bootstrap-grid.min.css`
  (grid + flex/display + spacing utilities); its components/utilities still in
  use are reimplemented with GC tokens in `static/css/gc-components.css`. The
  Bootstrap JS bundle is retained only for the remaining modal/collapse/alert
  widgets. **New or refactored pages should prefer `<gcds-grid>` /
  `<gcds-grid-col>` / `<gcds-container>` and `@cdssnc/gcds-utility` classes**
  over Bootstrap grid markup, and GCDS components over the `gc-components.css`
  shim, so the shim and the Bootstrap JS can eventually be removed.

## Development Notes

- Database changes require new Diesel migrations
- Templates use Tera syntax with Fluent filters for i18n
- GraphQL schema changes need corresponding resolver updates
- Static files are compiled at build time - restart after changes
- Email templates are in `templates/emails/` directory