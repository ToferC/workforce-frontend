# Org Chart Explorer — Implementation Plan

A read-only, visual, explorable org chart for an organization: click through org
tiers, expand down to the team level, and see the associated roles and people.
This complements (does not replace) the existing **org chart _builder_**
(`/{lang}/organization/{id}/org_chart`), which stays focused on editing.

## Decisions

1. **Separate read-only explore view** at
   `/{lang}/organization/{id}/org_chart/explore`. The builder is unchanged.
2. **HTML/CSS box tree** for rendering. *(Originally ECharts `tree`; pivoted —
   ECharts draws node-link graphs with text labels and can't render rectangular
   boxes with rich content inside, which a conventional org chart needs. The
   explorer is now a top-down "pure CSS tree" of boxes built by vanilla JS, with
   no new dependencies. The capacity heatmap is a coloured top border on team
   boxes; connector and accent colours come from the GC theme tokens.)*
3. **Single-query tier skeleton + lazy leaves.** The full tier+team skeleton is
   built server-side from one `OrgTiersByOrgId` call. Team **stats** (capacity,
   capability mix) and the **people** behind roles load lazily as the user
   drills in — appropriate for large organizations.
4. **Capacity heatmap.** Team/tier nodes are colored by active effort once their
   stats load, so the heatmap "fills in" as you expand.

## Data flow

```
explore.html (full page)
 ├─ ECharts tree  ← tier skeleton JSON (1× OrgTiersByOrgId, server-built nested)
 │     • tiers collapsed by default; ECharts only draws expanded subtrees
 │     • on tier EXPAND → lazy GET /org_tier/{id}/node.json
 │            → team stats + capacity-heatmap colors (setOption merge)
 │     • on team CLICK → HTMX GET /team/{id}/chart_panel
 └─ #tier-detail side panel ← team roles + people + work (1× TeamById)
```

## Reuse vs. new

| Reused (unchanged) | New |
|---|---|
| `OrgTiersByOrgId` (tiers + parent + team stubs) | `org_chart_explore` route/handler + `explore.html` |
| `OrgTierNode` (tier→teams→people→capabilities) | `org_tier_node_json` route/handler (JSON stats) |
| `TeamById` (roles, people, work, `capabilityCounts`) | `team_chart_panel` route/handler + `_team_panel.html` |
| `get_organization_by_id`, `require_role(User)` | `static/js/org_chart_explore.js` |
| Stat logic in `render_node` → extract `compute_team_stats()` | heatmap color scale in `theme.css` |
| `level_weight` / `domain_group` / `domain_short_label` | EN/FR i18n keys |
| viz macros, `themechange` event, chart-JSON pattern | |

**No GraphQL/schema changes required** — all three queries already exist with
the needed fields (`TeamById.capabilityCounts` gives a per-team capability
breakdown for the panel for free).

## Phases (each independently shippable)

- **Phase 1 — Visual tier skeleton (this PR).** `org_chart_explore` route + page +
  `org_chart_explore.js`. Server builds the nested tier+team JSON from one query;
  ECharts renders a collapsible/zoomable tree (org → tiers → teams). Entry points
  added from the organization and org-tier pages. No stats/panel yet.
- **Phase 2 — Capacity heatmap (done).** Rather than a separate lazy stats
  endpoint, the cheap server-computed aggregates `Team.headcount` and
  `Team.totalEffort` are added to the single `OrgTiersByOrgId` skeleton query, so
  the heatmap is baked into the initial render with no extra requests (people and
  capabilities — the expensive data — still stay lazy for Phase 3). Team nodes are
  colored by an effort band (empty / light / moderate / heavy) read from the GC
  theme tokens; tiers keep the neutral structural color; team labels show
  headcount; a legend is shown in the side panel.
- **Phase 3 — Role/people drill-down (done).** Each team box has a control that
  lazy-loads its roles and people as further boxes under it, from a new
  `team_members_json` endpoint (`/{lang}/team/{id}/members.json`, built from
  `TeamById`). Occupied roles link to the role and person; vacant roles are
  flagged. This is the expensive leaf data, fetched only when a team is expanded.
- **Phase 4 — Heatmap legend, a11y, polish.** Legend; `aria.enabled`; a
  "List view" toggle that swaps in the existing builder accordion as the
  keyboard/screen-reader path; dark-mode verification via `themechange`.

## Capacity heatmap

Reuse the band thresholds already in `render_node` (`>50` danger, `>20` warning,
else success) mapped to GC tokens (`--color-success/warning/danger`). Team node
fill = its band; tier node fill = rolled-up from its descendant teams (computed
client-side as data loads). Legend shows the three bands plus a neutral
"not loaded" state for unexpanded subtrees.

## Verification

Per phase: `cargo check`; Tera `Tera::new` parse; `fluent-syntax` parse of the
`.ftl` files; an ECharts render screenshot against sample tier JSON. The live
HTMX/API flow needs a deploy with DB + API to smoke-test.

## Risks

- **ECharts a11y** (canvas) — the Phase 4 list-view fallback is non-optional.
- **Very large orgs** — collapsed-by-default + draw-only-expanded bounds node
  count; cache per-tier stat fetches client-side so collapse/re-expand is free.
