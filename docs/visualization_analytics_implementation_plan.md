# Implementation Plan: Visualization & Ecosystem Analytics

## Goal

Build on the Phase 1–5 HR foundation to deliver two things:

1. **Pathway 1 — Visual understanding** of org structure, delivery priorities
   (Product → Task → Work), and organizational capability (Teams → People →
   Roles), so a manager can *look* at any team/product/org and immediately grasp
   who is there, how loaded they are, what they can do, and where the gaps are.
2. **Pathway 2 — Ecosystem-over-time analytics** that show the organization
   *learning and growing*: capability accumulation, talent mobility/promotion,
   and whether capability supply keeps pace with work demand.

This plan is frontend-focused. Items that require backend work are flagged
**[API]** and specified in detail in `docs/api_changes_for_analytics.md` for a
separate session/agent working on the `workforce_analytics` repo.

## Status (2026-06-15)

| Phase | Title | Status |
|-------|-------|--------|
| A | Visual vocabulary macros | ✅ Done |
| B | Vendor ECharts + scaffold | ✅ Done (CDN; vendor file pending offline) |
| C | Capability heatmap (Teams × Domains) | ✅ Done — `/{lang}/analytics/coverage` |
| D | Org-chart capability/capacity overlay | ✅ Done — team-node overlay |
| E | Delivery treemap (Product → Task → Work) | ✅ Done — `/{lang}/analytics/delivery` |
| F | Per-entity radar + requirement-match bars | ✅ Done — person & role pages |
| G | Talent mobility sankey | ✅ Done — `/{lang}/analytics/mobility` |
| H | Capability growth curve | ⛔ Blocked on **[API]** `capabilityGrowth` |
| I | Supply vs demand gap trend | ⛔ Blocked on **[API]** `capabilitySupplyDemand` |

Phases A–G ship against the **current** API with no backend changes. H and I
are gated on the aggregates in `docs/api_changes_for_analytics.md` (separate
`workforce_analytics` session). Notes per phase below record what was actually
built where it diverged from the original plan.

## Current state (baseline)

- Only visualization is the org-chart builder (`templates/org_chart/`): an
  HTMX-lazy-loaded `<details>` tree, structural only — no capability/capacity
  overlay.
- All detail pages are Bootstrap card/list/badge. **No charting library is
  loaded** (only Bootstrap 5, jQuery/jQuery-UI, Popper, HTMX).
- Phase 5 analytics dashboard (`/{lang}/analytics`, Analyst-gated) renders
  server-computed tables: work status, effort-by-domain, team capacity,
  over-allocation, capability gap. This is the natural home for new charts.
- History is **reconstructable but not stored**: no audit/snapshot table.
  `Role` (active+inactive, `startDate`/`endDate`/`rank`/`occupationalLevel`),
  `Validation.createdAt`+`validatedLevel`, `Capability.createdAt`,
  `Task.completedDate`, publication dates. `validatedLevel` is a recomputed
  average — historical capability must be rebuilt from individual `Validation`
  rows, not the current value. `Role` date fields are **String-typed**, not
  `NaiveDateTime`, and need parsing.

## Foundational decision: charting library

Adopt **Apache ECharts**, vendored like htmx at `static/echarts/echarts.min.js`
and loaded in `base.html`. One dependency-free file covers heatmap, treemap,
sunburst, radar, sankey, and graph — everything both pathways need. Chart.js
lacks treemap/sankey; D3 is too low-level for this team's velocity. Keep
server-rendered fallbacks (CSS heatmap, macro meters) so pages degrade
gracefully, consistent with the project's "must work as plain POST + redirect"
ethos. Charts read pre-aggregated JSON injected by handlers (the Phase 5
pattern) — no client-side GraphQL.

---

# Pathway 1 — Structure, priorities & capability

## Phase A — Visual vocabulary (Tera macros, no new deps)

**Objective:** Standardize the visual primitives so every page reads
consistently before any charts are added.

**Build (`templates/macros/viz.html`):**
- `effort_meter(value, ceiling=10)` — Bootstrap progress bar, green/amber/red,
  replacing bare `activeEffort` / `effort` integers.
- `capability_scale(self_level, validated_level)` — 5-step pip scale
  DESIRED→SPECIALIST; validated filled, self-identified outline.
- `status_chip(status)` and `domain_chip(domain)` — one canonical color map for
  `WorkStatus` (5) and `SkillDomain` (16).

**Retrofit:** `person.html`, `role.html`, `team.html`, `work.html`,
`task.html`, `product.html`, the Phase 5 dashboard, and the index tables to use
the macros.

**Dependencies:** none. **[API]:** none.

**Acceptance:** effort everywhere shows as a colored meter; capability levels
use one scale; domain/status colors identical across all pages.

## Phase B — Vendor ECharts + integration scaffold

**Objective:** Make charts available app-wide with a clean handler→template
contract.

**Build:**
- Vendor `static/echarts/echarts.min.js`; load in `base.html` (after htmx).
- `templates/macros/charts.html` — a `chart(id, height)` macro that emits a
  sized `<div>` + an init script reading a JSON `<script type="application/json">`
  payload by id. Establishes the "handler aggregates → template renders" pattern.
- Document the convention in CLAUDE.md (charts read injected JSON, never call
  GraphQL client-side).

**Dependencies:** Phase A helpful but not required. **[API]:** none.

**Acceptance:** a throwaway demo chart renders on the analytics page from
injected JSON; dark/light theme respected.

## Phase C — Capability heatmap (Teams × Domains)

**Objective:** The single highest-value new view — surface thin areas and
over-concentration at a glance.

**Build:**
- Grid: rows = teams (or org tiers), columns = 16 `SkillDomain`s, cell intensity
  = capability depth (count weighted by validated level; self-identified as
  fallback). **Server-rendered CSS table first** (no JS), ECharts heatmap as a
  progressive upgrade with drill-down to the team.
- New handler section on the analytics dashboard (or `/{lang}/analytics/coverage`).
- Reuses Phase 5's capability aggregation logic.

**Data:** prefer **[API]** `teamCapabilityMatrix` aggregate (see API doc) to
avoid fanning out across all people; until then, derive from
`analytics_people` + team grouping already added in Phase 5.

**Acceptance:** heatmap shows every team × domain with intensity; clicking a
cell links to the team; empty cells visibly flag coverage gaps.

**As built:** `/{lang}/analytics/coverage`. Depth = sum of capability level
weights (Desired 1 … Specialist 5, validated preferred) per team × domain,
derived frontend-side from `analytics_people` (the **[API]**
`teamCapabilityMatrix` aggregate was *not* required). ECharts heatmap plus a
server-rendered CSS-opacity fallback table and a domain-strength ranking.
Domains with zero coverage are dropped from the axes.

## Phase D — Org-chart capability & capacity overlay

**Objective:** Make the existing structural tree answer "where is the org
strong / stretched / hollow?"

**Build (in `templates/org_chart/chart_macros.html`, `node.html`, `panel.html`):**
- Per team/tier node: **vacancy ratio** badge (occupied vs vacant roles — data
  already present), **aggregate load** mini-meter (rolled-up effort), and a thin
  **domain-mix** stacked bar.
- Panel: add capability summary + capacity for the selected tier.

**Data:** extend the org-chart handler's GraphQL to pull role effort + team
capability counts per node. **[API]:** optional `OrgTier.capabilityCounts` /
`Team.capabilityCounts` rollups would simplify (see API doc); otherwise compute
frontend-side from existing nested fields.

**Acceptance:** every tree node shows vacancy %, load, and domain mix without
expanding; overloaded/hollow pockets are visible at the structural level.

**As built:** the overlay lives in the lazy-loaded team node (not the tier node
or panel). Each team card shows its top-3 capability domains as coloured
`domain-chip` badges plus a total-effort badge (green/amber/red by load). The
existing `org_tier_node.graphql` query was extended with `person.activeEffort`
and `person.capabilities`, so the overlay arrives with the existing HTMX
fetch — no new endpoint, no **[API]** rollups. Tier-level rollups and the panel
summary remain a possible follow-on.

## Phase E — Delivery / priorities flow (Product → Task → Work)

**Objective:** Show where the org's effort actually goes and what's blocked.

**Build:**
- ECharts **treemap** (effort = area, status = color) and/or **sunburst** of
  Product → Task → Work.
- Add to `product.html` (scoped to one product) and a new org-wide
  `/{lang}/analytics/delivery` page.

**Data:** `all_work` already returns task + effort + status; add product/task
rollup in the handler. **[API]:** none required; an aggregate
`deliveryRollup` query is a nice-to-have for large datasets.

**Acceptance:** treemap renders the full delivery tree; blocked/at-risk work is
color-visible; effort concentration is obvious; drill-down links to entities.

**As built:** org-wide `/{lang}/analytics/delivery` ECharts treemap (rectangle
area = work effort, leaf colour = work status) with zoom-to-node + breadcrumb.
A single nested `delivery_treemap.graphql` query (`allProducts → tasks → work`)
feeds it; the handler pre-computes every node's value so parents size correctly
even when a task has no work. KPI cards + a Products-by-Effort table round it
out. The per-product treemap on `product.html` was deferred (the org-wide view
covers the headline need).

## Phase F — Per-entity enrichments

**Objective:** Richer single-entity comprehension.

**Build:**
- **Person / Role:** capability **radar** across the 16 domains; on a role, a
  requirement-vs-capability **match bar** (reuse `capabilityMatches`).
- **Team:** capacity + domain-coverage summary card at top of `team.html`.

**Dependencies:** Phases A, B. **[API]:** none.

**Acceptance:** person/role pages show a domain radar; role shows per-requirement
fill vs the assignee/candidate; team shows a coverage summary.

**As built:** person page shows a capability radar (validated filled +
self-identified dashed), rendered only when the person has ≥3 distinct domains
(a radar needs ≥3 axes). Role page shows requirement-match bars (Required vs
Held, green when the incumbent meets/exceeds, red when short); `role_by_id`
now returns the incumbent's capabilities so no extra round trip is needed. The
team-page coverage summary card was deferred (Phase C already gives the
team × domain picture org-wide).

---

# Pathway 2 — Ecosystem over time

## Phase G — Talent mobility & promotion flow (Sankey) — *no API dependency*

**Objective:** Visualize "people move across the org, are promoted."

**Build:**
- ECharts **sankey**: team→team transitions and level→level promotions
  (`rank` for military, `occupationalLevel` for civilian) over a selectable
  window, plus inflow (new) and outflow (`retiredAt`).
- New `/{lang}/analytics/mobility` section.
- New query `queries/analytics/analytics_role_history.graphql` pulling, per
  person, active+inactive roles with `startDate`/`endDate`/`team`/`rank`/
  `occupationalLevel`.

**Data/logic:** sort each person's roles by `startDate`; diff team & level
between consecutive roles to derive transitions. **Caveat:** parse String-typed
`Role` dates defensively. Doable **entirely frontend-side today**.

**Acceptance:** sankey shows movements for the chosen period; node = team or
level; flows quantify promotions/laterals/inflow/outflow.

**As built:** `/{lang}/analytics/mobility` ECharts sankey. To stay within
ECharts' no-cycle constraint (team A→B and B→A would error), the graph is
bipartite: each person's most-recent prior team **(was)** flows to their
current team **(now)**, weighted by headcount. Query `analytics_mobility.graphql`
pulls active + inactive roles with teams and dates; the handler picks the
latest-start active role and latest-end inactive role per person. A sorted
transition table backs it up. **Caveat noted:** `Role` dates are String-typed
and compared lexically — correct for ISO-8601 but not validated as such. The
richer level→level promotion flow and a time-window selector are follow-ons.

## Phase H — Organizational capability growth ("learning curve") — **[API]**

**Objective:** The clearest answer to "the org learns and grows."

**Build:**
- ECharts cumulative **stacked area / line** of validated capability over time,
  split by `SkillDomain`.
- New `/{lang}/analytics/growth` section.

**Data:** rests on the only truly timestamped growth signal —
`Validation.createdAt` + `validatedLevel`. **[API] required:** there is no
top-level `allValidations`; org-wide reconstruction by fanning out across all
people will not scale. Needs aggregate `capabilityGrowth(bucket, domain)` query
(see API doc). Always reconstruct historical level from individual validations,
never from the current `Capability.validatedLevel`.

**Acceptance:** chart shows cumulative validated capability per domain by month;
buckets and domain filter selectable; loads from a single aggregate query.

## Phase I — Capability supply vs demand gap trend — **[API]**

**Objective:** The most actionable planning view — is capability growth keeping
pace with work taken on?

**Build:**
- Two lines per domain over time — **supply** (validated capabilities) vs
  **demand** (`Requirement`s + `Work.capabilityLevel`) — with the gap shaded.
  The longitudinal version of Phase 5's static gap table.
- Add to `/{lang}/analytics/growth` or its own section.

**Data:** supply from Phase H pipeline; demand from requirement/work creation
timestamps. **[API] required:** aggregate `capabilitySupplyDemand(bucket,
domain)` (see API doc).

**Acceptance:** per-domain supply & demand lines with shaded gap; widening gaps
visibly flag emerging shortfalls.

---

## Honorable mention (building block, not scheduled)

Per-person **career timeline** (Gantt-style role swimlane with promotion
markers) — concrete and great for talent management, but individual rather than
ecosystem-level, and effectively the unit Phase G aggregates. Easy follow-on to
Phase G using the same `analytics_role_history` query.

## Recommended sequence

1. **A → B → C** — high value, mostly server-side; lands ECharts.
2. **D** — leverages the new overlay primitives.
3. **G** — first time-series win, **no API work needed**.
4. **E, F** — richer structure/entity views.
5. **H, I** — gated on the API aggregates; kick off the API work (separate
   session, `docs/api_changes_for_analytics.md`) in parallel with steps 1–3 so
   the aggregates are ready by the time the frontend reaches them.

## Cross-cutting notes

- New analytics sections live under the existing `/{lang}/analytics` dashboard,
  Analyst-gated via `security::require_role(.., MinimumRole::Analyst)`.
- Handlers aggregate; templates render injected JSON (Phase 5 pattern). Inject
  chart option JSON through `crate::chart_json()`, which escapes `<` so a
  `</script>` inside user-controlled labels can't break out of the inline
  `<script type="application/json">` block (stored-XSS / chart-breakage guard).
- History is derived by bucketing `createdAt`/`startDate` into months — good for
  trends, not exact point-in-time audit.
- Keep `schema.graphql` in sync with `workforce_analytics` whenever **[API]**
  items land.
