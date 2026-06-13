# Frontend CRUD Plan: Epifront ↔ workforce_analytics

Plan for adding create / update / delete (retire) functionality to the
workforce-frontend (Epifront) app against the workforce_analytics GraphQL API.

## 1. Where things stand

### Backend API (workforce_analytics)

The API already exposes everything needed — no backend work is required to
start:

- **Create + update mutations exist for every domain entity**: organizations,
  org_tiers, org_ownerships, persons, roles, teams, team_ownerships,
  affiliations, skills, capabilities, requirements, validations, language_data,
  publications, publication_contributors, tasks, work, plus `createUser` /
  `updateUser` / `signIn`.
- **There are no delete mutations.** Deletion is a soft-delete: most update
  inputs accept an optional `retiredAt` timestamp (`OrganizationData`,
  `OrgTierData`, `TeamData`, `PersonData`, `CapabilityData`,
  `RequirementData`, …). Roles are "deleted" by setting `active: false` +
  `endDate`; affiliations by setting `endDate`.
- **Auth**: JWT bearer token from the `signIn` mutation. All mutations are
  guarded — `Operator` role or above for most, `Admin` for user and validation
  mutations. `User`/`Analyst` roles are read-only.
- The full schema lives at `workforce_analytics/schema.graphqls` and the
  mutation resolvers in `graphql_api/src/graphql/mutation/`.

### Frontend (this repo)

- Pure thin client: no local DB; all data via `graphql_client` (0.14) POSTs to
  `GRAPHQL_API_TARGET`. Bearer token + role/user_id stored in the cookie
  session at login (`src/handlers/authentication_hander.rs:59-80`).
- **Every entity route today is read-only GET** (`src/handlers/routes.rs`).
  The only mutation wired up is `logIn`.
- `src/handlers/role.rs:51-90` contains stub `create_role` / `role_submit`
  handlers (not registered in routes, no mutation behind them) — the intended
  pattern is already sketched: Tera form → `web::Form` POST → GraphQL mutation.
- Stack: Actix-Web 4.11, Tera + Fluent (EN/FR), Bootstrap 5, jQuery loaded but
  essentially unused. No HTMX, no WASM.
- Role checks exist only in templates (`base.html` checks `role == "admin"`);
  handlers do not enforce anything.

## 2. Technology decision: HTMX over WASM

**Recommendation: keep server-side Tera rendering and layer HTMX on top.
Do not adopt WASM.**

Why not WASM (Yew/Leptos/Dioxus):

- It replaces the view layer rather than extending it — the existing Tera
  templates, Fluent i18n filter, cookie-session auth, and URL-based language
  switching would all need re-implementation inside the WASM app.
- It splits the codebase into two frontends (SSR pages + WASM islands) with
  duplicated GraphQL types, doubling maintenance for a CRUD-forms app that
  gets no benefit from client-side state.
- Build/deploy gets heavier (wasm-pack/trunk toolchain, larger payloads),
  while the actual requirement — forms that call mutations — is the thing
  server-side Rust already does best here.

Why HTMX fits:

- The handler/template pattern stays exactly as-is; HTMX endpoints are just
  more Actix handlers that render Tera *partials* instead of full pages.
- Forms work without JavaScript first (plain POST + redirect), then HTMX
  upgrades them: inline validation errors, dependent `<select>`s, modals,
  retire-with-confirm — all returning server-rendered HTML, so Fluent i18n
  keeps working.
- One new `<script>` tag vendored into `static/` (compiled into the binary by
  `build.rs`); jQuery can eventually be dropped.

## 3. Implementation plan

### Phase 0 — Foundations (do once, everything else builds on it) ✅ DONE

1. ✅ **Shared GraphQL helper.** `post_graphql<Q: GraphQLQuery>` in
   `src/graphql/client.rs` with an `ApiError` enum that surfaces GraphQL
   `errors[]` instead of `.expect()` panics. All entity modules migrated.
   This also fixed a latent auth bug: the old code sent the JWT in a header
   literally named `Bearer`, but the API only reads `Authorization: Bearer
   <token>` — no frontend request was actually authenticated before.
2. ✅ **Sync the schema.** It turned out the *frontend's* `schema.graphql` was
   current and the backend's checked-in `schema.graphqls` was stale (missing
   `UserResponse.id` / `expiresAt`); the backend file was updated to match the
   code. Sync rule documented in CLAUDE.md.
3. ✅ **Handler-level authorization.** `security::require_role(&session, &lang,
   MinimumRole::Operator)` redirects to log-in on missing/expired session and
   to `/not_authorized` on insufficient role.
4. ✅ **Feedback + safety plumbing:** session flash messages
   (`security::add_flash`, rendered as alerts in `base.html`), CSRF helpers
   (`get_or_create_csrf_token` / `verify_csrf_token`, token injected into
   every context by `generate_basic_context`), and shared Bootstrap form
   macros in `templates/macros/forms.html`.
5. ✅ **Vendor HTMX** — htmx 2.0.4 at `static/htmx/htmx.min.js`, loaded in
   `base.html`.

### Phase 1 — Pilot entity: Organization (establish the full pattern) ✅ DONE

Implemented as planned (mutation queries, handlers, shared form template,
retire-confirm page, EN/FR strings, operator/admin-only buttons). Notes:

- Verified end-to-end against a locally running API: create → edit →
  CSRF rejection → retire round trip as admin; unauthenticated requests
  redirect to log-in.
- Template render tests added in `tests/templates_render.rs` (run with
  `cargo test`) — they catch Tera runtime errors the startup parse doesn't.
- Two cross-cutting fixes landed here: session roles are normalized to
  lowercase at login (the API returns "ADMIN", templates compare "admin"),
  and URL strings built with Tera `~` concatenation need `| safe` to avoid
  HTML-escaping of slashes.

Organization is the simplest entity (no foreign keys in its create input).
Build the complete vertical slice once, then copy it:

- Queries: `queries/organizations/create_organization.graphql`,
  `update_organization.graphql` (update also serves retire via `retiredAt`).
- `src/graphql/organization.rs`: `CreateOrganization` / `UpdateOrganization`
  `GraphQLQuery` structs + thin wrapper fns.
- Handlers in `src/handlers/organization.rs`, registered in `routes.rs`:
  - `GET  /{lang}/organization/new` → form
  - `POST /{lang}/organization/new` → createOrganization → redirect to detail
  - `GET  /{lang}/organization/{id}/edit` → form pre-filled from
    `organizationById`
  - `POST /{lang}/organization/{id}/edit` → updateOrganization
  - `POST /{lang}/organization/{id}/retire` → updateOrganization with
    `retiredAt = now()`, behind a confirm step
- Templates: `organization/organization_form.html` (shared create/edit via
  the form macros), confirm-retire partial; Edit/Retire buttons on the detail
  page and index, shown only for operator/admin.
- Fluent strings for every new label/button/flash in both
  `i18n/en/epifront.ftl` and `i18n/fr/epifront.ftl`.
- HTMX: form posts with `hx-post` returning the form partial with inline
  errors on failure; retire button opens an HTMX confirm modal.

Exit criteria: create → edit → retire round-trip works against a locally
running workforce_analytics API as an operator, and is correctly refused
(403 page / hidden buttons) as a plain user.

### Phase 2 — Core org-structure entities (IN PROGRESS)

Same slice, in dependency order, reusing the macros and helper:

| Entity | Notes specific to its forms |
|---|---|
| **OrgTier** ✅ | Done, including the **org chart builder** (`/{lang}/organization/{id}/org_chart`): two-pane view with HTMX lazy-loaded tier tree (tiers → teams → roles → people), info panel, and inline add-child-tier. See `docs/ORG_CHART_BUILDER.md` and the mockup in `docs/mockups/`. Two backend bugs found and fixed: `OrgTier.owner` panic for ownerless tiers, and `SkillDomain` schema drift (16 live values vs 13 stale). |
| **Team** ✅ | Done, including inline "+ add team" in the org chart builder (same HX-Retarget pattern as tiers). The API can't move a team to another tier after creation, and `Team` doesn't expose its current domain — the edit form offers "keep current domain". Two more backend bugs fixed: `Team.owner` unwrap panic for ownerless teams (now inherits the tier's owner) and a slice panic on malformed Authorization headers. |
| **Person** ✅ | Done. Create resolves the linked user account by email via `userByEmail` (one person per user — duplicates rejected cleanly). Backend bug fixed: `updatePerson` mutated the struct but never saved it (and skipped `country`) — every person edit was silently discarded. |
| **Role** ✅ | Done. Stub handlers rewritten to match the API's `NewRole` (team, titles, effort, occupation, rank, start date). `updateRole` only accepts `active`/dates **by design** (history preservation), so the edit UI is a status form (active flag + dates) plus an "End role" action — not a free edit; titles/assignment change by creating a new role. Vacant roles omit `personId`; assignment is by typed full name resolved against `personByName`. "+ add role" wired into the builder's team nodes and the team page. Note: `personByName` does `ilike` on family **or** given name separately and can't match a "Given Family" string, so the handler searches the last name token and filters for an exact full-name match. |
| **OrgOwnership / TeamOwnership / Affiliation** ✅ | Done. "Assign owner" actions on the tier and team pages create an ownership record (resolving the person by typed full name via the shared `resolve_person_by_name` helper) — this is what lets a tier/team have a real owner instead of the API's parent-chain inheritance fallback. "Add affiliation" on the person page (org select + role; `homeOrgId` defaults to the person's own org) plus per-affiliation "End" (sets `endDate`). Also fixed a latent template typo (`person.affilations`) that had been hiding the affiliations list entirely. **Known gap:** `updateOrgOwnership`/`updateTeamOwnership` need the ownership record's id, which no query exposes, so *changing* an existing owner isn't possible yet — only assigning one (see backend gaps §5). |

### Phase 3 — Skills & capabilities

| Entity | Notes |
|---|---|
| **Skill** | Plain bilingual form + `domain` enum select. ⚠️ `SkillData` has **no** `retiredAt` — skills can't be retired through the API today (backend gap, §5). |
| **Capability** | Created from a person's page (person + skill + org + self-identified level). Update = level changes + retire. HTMX inline level editing on the person page is the highest-value enhancement here. |
| **Requirement** | Created from a role's page (skill + required level). |
| **Validation** | Admin-only (matches backend guard); simple level form on the capability page. |
| **LanguageData** | Section on the person page; Canadian A/B/C/E/X level selects. |

### Phase 4 — Work-tracking & publications

Tasks, Work, Publications, PublicationContributors — same pattern, enum-heavy
forms (`TaskStatus`, `WorkStatus`, `PublicationStatus`, `CapabilityLevel`).
Lower priority; schedule after Phases 1–3 are proven.

### Phase 5 — List/index pages & polish

- Today most entities have only detail pages reachable by ID. Add index pages
  (the queries exist: `allOrganizations`, `allTeams`, `allRoles`, `skills`,
  `allPeople`, `allTasks`, …) with New/Edit/Retire actions — otherwise the
  CRUD UI has no entry point.
- HTMX-powered search/filter on index pages (server-rendered table partials).
- Hide retired records by default with a "show retired" toggle; offer
  "restore" (update with `retiredAt: null`) where the API allows it.

## 4. Cross-cutting conventions

- **Route shape**: `GET/POST /{lang}/{entity}/new`,
  `GET/POST /{lang}/{entity}/{id}/edit`, `POST /{lang}/{entity}/{id}/retire`.
- **Delete is always soft** and always behind a POST + confirm — never a GET.
- **Enums in forms**: `graphql_client` generates Rust enums from the schema;
  derive `strum::EnumIter`-style listings (or hand-maintained lists in one
  module) to feed `<select>` options, with Fluent keys per variant for EN/FR
  labels.
- **Progressive enhancement**: every form must work as a plain POST +
  redirect; HTMX attributes are an overlay, not a requirement.
- **Verification per phase**: `cargo check` + manual round-trip against a
  local backend (`docker compose up` in workforce_analytics, seed dummy data,
  sign in as admin from `.env`).

## 5. Backend gaps to address in workforce_analytics (separate, small PRs)

Not blockers for Phases 1–2, but worth fixing alongside:

1. `SkillData` lacks `retiredAt` → skills cannot be retired via the API.
2. `NewAffiliation` lacks `startDatestamp` even though `Affiliation` has it.
3. No restore semantics documented — confirm `retiredAt: null` un-retires, or
   add explicit restore mutations.
4. `updatePublication` cannot change `leadAuthorId`; `updateUser` exists but
   there is no way to deactivate a user — consider an `active`/`retiredAt`
   field on users.
5. Optional QoL: `skills`/`allCapabilities`-style unpaginated list queries are
   inconsistent (some domains have `all*`, some only `*(count)`).

## 6. Suggested sequencing

1. Phase 0 + Phase 1 (Organization slice) — one PR; establishes every pattern.
2. Phase 2 (OrgTier, Team, Person, Role + link entities) — one PR per entity
   or small groups.
3. Phase 3 (skills/capabilities) and the backend-gap PRs in parallel.
4. Phases 4–5 as capacity allows.
