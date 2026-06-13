# Org Chart Builder

A two-pane visual builder for an organization's structure: **info on the
left** (selected tier details + actions), **the expandable chart on the
right** (tiers → teams → roles → people). Static design mockup:
`docs/mockups/org_chart_builder.html`. Live v1:
`/{lang}/organization/{id}/org_chart`.

## What v1 implements

| Piece | Route | Renders |
|---|---|---|
| Builder page | `GET /{lang}/organization/{id}/org_chart` | `org_chart/builder.html` — left info panel + collapsed root tiers |
| Node body (lazy) | `GET /{lang}/org_tier/{id}/node` | `org_chart/node.html` — child tiers, teams with occupied/vacant roles + people |
| Info panel | `GET /{lang}/org_tier/{id}/panel` | `org_chart/panel.html` — details, owner, edit/retire actions |
| Inline add form | `GET /{lang}/org_tier/new` (with `HX-Request`) | `org_chart/add_tier_form.html` |
| Create tier | `POST /{lang}/org_tier/new` | HTMX: re-rendered parent node; plain: redirect to tier page |
| Inline add team | `GET/POST /{lang}/team/new` (with `HX-Request`) | `org_chart/add_team_form.html`, same swap pattern |
| Inline add role | `GET/POST /{lang}/role/new` (with `HX-Request`) | `org_chart/add_role_form.html`, on team nodes; success retargets the tier node body |

Tier CRUD (full pages, work without JS) lives alongside:
`/{lang}/org_tier/new`, `/{id}/edit`, `/{id}/retire`.

## HTMX wiring

The whole builder is server-rendered Tera partials; htmx (vendored at
`static/htmx/htmx.min.js`) provides five behaviours:

1. **Lazy expansion.** Each tier renders as a `<details>` element whose
   body is a placeholder `div` with `hx-get="…/node"` and
   `hx-trigger="intersect once"`. Opening the `<details>` makes the
   placeholder visible, the intersection observer fires, and the node
   content loads exactly once. Children render collapsed with their own
   lazy bodies, so deep charts cost nothing until explored.
2. **Info panel.** The ℹ️ button on every node does
   `hx-get="…/panel" hx-target="#info-panel"`, with
   `onclick="event.preventDefault()"` so it doesn't toggle the
   `<details>`. The panel is sticky-positioned on the left.
3. **Inline add forms with content negotiation.** The "+ add child tier"
   button is a real link to the full-page form (no-JS fallback) *and* has
   `hx-get` to the same URL. The handler checks the `HX-Request` header
   and returns the compact form partial instead of the full page.
4. **Server-controlled swap on success.** The inline form posts with
   `hx-target="this" hx-swap="outerHTML"`, so validation errors simply
   re-render the form in place. On success the handler re-renders the
   *parent's* node body and redirects the swap with response headers:
   `HX-Retarget: #node-body-{parent_id}` + `HX-Reswap: innerHTML` —
   the new child appears in the tree without touching the rest of the
   page. Root-tier additions return `HX-Redirect` for a full reload.
5. **CSRF.** Partials are rendered through `generate_basic_context`, so
   the session's `csrf_token` is available and embedded as a hidden
   field; the POST handler validates it the same as plain forms.

## API calls used (all exist today)

- `organizationById` — chart header.
- `orgTiersByOrgId` — root tiers (filtered to `parentTier == null`)
  and parent-select options.
- `orgTierById` — info panel (details, owner, counts).
- **`OrgTierNode`** (new query document, existing schema fields) —
  `orgTierById { childOrganizationTier, teams { occupiedRoles { person },
  vacantRoles } }`. The schema's nested fields meant the read side needed
  **zero API changes**.
- `createOrgTier` / `updateOrgTier` — create, edit, retire (and re-parent
  via `parentTier`).

## API bugs found and fixed while building (workforce_analytics branch)

1. **`OrgTier.owner` panicked the API worker** (`models/org_tier.rs:58`
   `.unwrap()`) for any tier without an `OrgOwnership` record — which is
   every tier created through `createOrgTier`, since `NewOrgTier` has no
   owner field. Repeated panics wedged the whole server. Fixed: the
   resolver now walks up the parent chain and inherits the nearest
   ancestor's owner; ownerless root tiers return a GraphQL error instead
   of crashing.
2. **`SkillDomain` schema drift.** Both repos' schema files carried an
   old 13-value enum; the API actually serves 16 different values
   (SOFTWARE_ENGINEERING, CYBER_SECURITY, DATA_ANALYTICS_AND_AI, …).
   Mutations with stale values were rejected at runtime. Both schema
   files and the frontend's domain select are now synced from live
   introspection. **Process note:** the running API's introspection is
   the source of truth; `schema.graphqls` is not generated automatically
   and drifts.
3. **`Team.owner` had the same unwrap panic** (`models/team.rs:197`)
   for teams without a `TeamOwnership` record — every team created via
   `createTeam`. Fixed: falls back to the owning tier's (inherited)
   owner.
4. **Malformed Authorization header panicked the worker**
   (`models/auth.rs:65`): a header shorter than `"Bearer "` made the
   slice index out of bounds. Fixed with a length guard.
5. **`updatePerson` never persisted.** The mutation copied fields onto
   the loaded struct and returned it without calling `person.update()`
   (it also ignored `country`), so every person edit silently succeeded
   without saving. Fixed.

## API additions needed for the full builder vision

In rough priority order:

1. **Tier owner management.** `NewOrgTier` should accept an optional
   `ownerId` (creating the `OrgOwnership` row), and there should be an
   `orgOwnershipByTierId` query — today there is no way to *find* the
   ownership record's id, so `updateOrgOwnership` (change owner) is
   unusable from a client. Until then, new tiers inherit their parent's
   owner for display.
2. **Move roles between teams.** `RoleData` has no `teamId`, so
   drag-to-move a role (mockup's ⠿ handle) can't be implemented. By
   design history says "create a new role instead" — if that stands, the
   builder needs a `transferRole(roleId, newTeamId)` convenience mutation
   that closes the old role and opens a copy.
3. **Promote a tier to root.** `OrgTierData.parentTier` uses
   "absent = unchanged" semantics, so `parentTier` can never be cleared.
   Needs explicit null support (e.g. a `clearParentTier: Boolean` flag).
4. **Structured `orgChart` query.** `orgChart(id)` currently returns
   `[String!]!`. A structured tree (or flat list with parent ids) in one
   call would allow whole-chart export/print and avoid per-node loads
   for small organizations.
5. **Add team / add role from the builder** (frontend work, mutations
   exist): `createTeam` needs organization + tier (both known in
   context); `createRole` needs team + title + effort + occupation +
   rank + start date — the inline form should offer sensible defaults.
   This is the next Phase 2 slice.

## Follow-the-pattern checklist for the next entity slices

Each new builder action follows the tier pattern: full-page form first
(no-JS fallback), `HX-Request` content negotiation for the inline
variant, `hx-target="this"` for error re-render, `HX-Retarget` to the
affected branch on success, fluent strings in EN/FR, and a render test
in `tests/templates_render.rs`.
