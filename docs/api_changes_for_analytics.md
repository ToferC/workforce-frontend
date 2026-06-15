# API Change Spec: Ecosystem Analytics Aggregates

**Audience:** A separate session/agent working on the **`workforce_analytics`**
repo (the GraphQL API; `schema.graphqls` there is the source of truth and must
stay in sync with `schema.graphql` in `workforce-frontend`).

**Why:** The frontend visualization/analytics plan
(`docs/visualization_analytics_implementation_plan.md`) needs server-side
aggregates. The current schema exposes per-entity data only; the time-series and
matrix views cannot be assembled efficiently on the client because:

- There is **no top-level `allValidations`** query — validations are nested
  under `Person.validations` / `Capability.validations`. Reconstructing org-wide
  capability growth would require fanning out across every person and every
  capability. This will not scale.
- `Capability.validatedLevel` is a **recomputed average**, so historical levels
  must be derived from individual `Validation.createdAt` rows, not the current
  field. That reconstruction belongs server-side.
- There is no aggregate for Team × Domain capability depth.
- `Role` date fields (`startDate`, `endDate`, `createdAt`, `updatedAt`) are
  **String-typed**, not `NaiveDateTime`, which complicates time bucketing.

Each item below is independent and can be delivered/merged separately. Items are
ordered by frontend priority (P1 unblocks the most-valued views).

---

## Conventions for all new queries

- All new queries are **read-only** and must respect the existing
  `user < analyst < operator < admin` authorization. Gate at **analyst+**
  (these are management analytics).
- A shared time-bucket argument:
  ```graphql
  enum TimeBucket { WEEK, MONTH, QUARTER, YEAR }
  ```
- Time-series points use a stable shape:
  ```graphql
  type TimeSeriesPoint {
    "Start of the bucket, ISO 8601"
    periodStart: NaiveDateTime!
    bucket: TimeBucket!
    value: Float!
  }
  ```
- Where a series is split by category (domain/level), return a labeled series:
  ```graphql
  type LabeledSeries {
    "e.g. a SkillDomain or CapabilityLevel value, as a String"
    key: String!
    points: [TimeSeriesPoint!]!
  }
  ```
- Buckets must be **dense** (emit zero-value points for empty periods in range)
  so the frontend can render continuous lines without gap-filling.
- Cumulative vs incremental: where noted "cumulative", `value` is the running
  total as of `periodStart`.

---

## P1 — `capabilityGrowth` (unblocks Phase H: learning curve)

Cumulative validated capability over time, split by domain. Drives the
org "learning curve".

```graphql
extend type Query {
  """
  Cumulative validated capability across the organization over time,
  one series per SkillDomain. A capability counts toward a period once it
  has a validation at or before that period's end; weight by validated level.
  Reconstruct historical validated_level from Validation records
  (createdAt + validatedLevel), NOT from Capability.validatedLevel.
  """
  capabilityGrowth(
    bucket: TimeBucket!
    "Inclusive lower bound; if null, earliest validation."
    from: NaiveDateTime
    "Inclusive upper bound; if null, now."
    to: NaiveDateTime
    "Optional filter to a single domain; if null, all domains as separate series."
    domain: SkillDomain
    "Optional filter to an org subtree."
    orgTierId: UUID
  ): [LabeledSeries!]!   # key = SkillDomain
}
```

**Semantics / weighting:** `value` = sum over capabilities of a level weight
(DESIRED=0, NOVICE=1, EXPERIENCED=2, EXPERT=3, SPECIALIST=4) using the validated
level **as of** the bucket end, reconstructed from that capability's validations.
Cumulative. Exclude `retiredAt`-flagged capabilities once retired.

**Acceptance:** for a known fixture, the latest bucket equals the current
org-wide validated-capability weight per domain; earlier buckets are monotonic
non-decreasing (cumulative) except where capabilities retire.

---

## P1 — `capabilitySupplyDemand` (unblocks Phase I: gap trend)

Supply vs demand per domain over time.

```graphql
type SupplyDemandPoint {
  periodStart: NaiveDateTime!
  bucket: TimeBucket!
  "Count/weight of validated capabilities available as of period end."
  supply: Float!
  "Count/weight of required capability from role Requirements + Work as of period end."
  demand: Float!
}

type SupplyDemandSeries {
  "SkillDomain value as String."
  domain: String!
  points: [SupplyDemandPoint!]!
}

extend type Query {
  """
  Per-domain capability supply (validated capabilities) vs demand
  (active role Requirements + active Work.capabilityLevel) over time.
  Demand items count from their createdAt; supply reconstructed from
  Validation history as in capabilityGrowth.
  """
  capabilitySupplyDemand(
    bucket: TimeBucket!
    from: NaiveDateTime
    to: NaiveDateTime
    domain: SkillDomain
    orgTierId: UUID
  ): [SupplyDemandSeries!]!
}
```

**Semantics:** supply identical weighting to `capabilityGrowth`. Demand =
weighted count of `Requirement.requiredLevel` plus `Work.capabilityLevel` for
work that is active (not COMPLETED/CANCELLED) as of the bucket. Both cumulative
snapshots at period end so the gap = demand − supply is directly chartable.

**Acceptance:** latest bucket matches the Phase 5 static gap table totals per
domain (sanity cross-check against the existing frontend computation).

---

## P2 — `teamCapabilityMatrix` (unblocks Phase C: heatmap; helps Phase D)

Team × Domain capability depth, in one call.

```graphql
type TeamCapabilityCell {
  domain: String!          # SkillDomain value
  "Weighted capability depth (validated; self-identified fallback)."
  depth: Float!
  "Distinct people contributing at this domain."
  peopleCount: Int!
}

type TeamCapabilityRow {
  teamId: UUID!
  teamName: String!
  cells: [TeamCapabilityCell!]!   # one per SkillDomain present; absent = 0
}

extend type Query {
  "Capability depth per team across all skill domains (for a heatmap)."
  teamCapabilityMatrix(orgTierId: UUID): [TeamCapabilityRow!]!
}
```

**Acceptance:** sum of `peopleCount` reconciles with team membership; depth uses
validated level, falling back to self-identified where unvalidated.

---

## P2 — Node capability rollups (helps Phase D: org-chart overlay)

Add capability/effort rollups directly to hierarchy types so the org chart can
render overlays without extra round-trips.

```graphql
extend type Team {
  "Capability counts for people holding roles in this team."
  capabilityCounts: [CapabilityCount!]!
  "Sum of active effort across this team's roles."
  totalEffort: Int!
}

extend type OrgTier {
  "Capability counts rolled up across this tier and its descendants."
  capabilityCounts: [CapabilityCount!]!
  "Sum of active effort across this tier and descendants."
  totalEffort: Int!
}
```

`CapabilityCount` already exists `{ name, domain, level, counts }`. **Acceptance:**
tier rollups equal the sum of descendant teams; effort matches the sum of role
efforts.

---

## P3 — `talentMovements` (optional server version of Phase G)

Phase G (mobility sankey) can be done **frontend-only** from role history, so
this is optional — provide it only if frontend-side reconstruction proves too
heavy.

```graphql
type TalentMovement {
  personId: UUID!
  at: NaiveDateTime!
  fromTeamId: UUID        # null = inflow / new hire
  toTeamId: UUID          # null = outflow / retirement
  fromLevel: String       # rank or occupationalLevel as String
  toLevel: String
  "PROMOTION | LATERAL | INFLOW | OUTFLOW"
  kind: String!
}

extend type Query {
  "Derived role transitions over a window, for mobility/promotion analysis."
  talentMovements(from: NaiveDateTime, to: NaiveDateTime, orgTierId: UUID): [TalentMovement!]!
}
```

**Derivation:** sort each person's roles by start; emit a movement per
consecutive pair (team and/or level change), plus inflow at first role and
outflow at `retiredAt`.

---

## P3 — Data-quality fix: normalize `Role` date types

`Role.startDate`, `endDate`, `createdAt`, `updatedAt` are currently **`String!`**.
Every time-series view that touches role history must parse these. Strongly
recommend exposing proper `NaiveDateTime` variants (keep the String fields for
backward compat, or migrate with a deprecation):

```graphql
extend type Role {
  startDatestamp: NaiveDateTime
  endDatestamp: NaiveDateTime
}
```

**Acceptance:** new fields parse 1:1 with the existing strings; `null` where the
string is empty/unset.

---

## Delivery checklist for the API agent

- [ ] **P1** `capabilityGrowth` + `TimeBucket`, `TimeSeriesPoint`, `LabeledSeries`
- [ ] **P1** `capabilitySupplyDemand` + `SupplyDemandPoint`/`SupplyDemandSeries`
- [ ] **P2** `teamCapabilityMatrix` + `TeamCapabilityCell`/`TeamCapabilityRow`
- [ ] **P2** `Team.capabilityCounts` / `Team.totalEffort` / `OrgTier.*` rollups
- [ ] **P3** `talentMovements` (optional)
- [ ] **P3** `Role.startDatestamp` / `endDatestamp`
- [ ] Authorization: analyst+ on all new queries
- [ ] Dense buckets (zero-fill empty periods)
- [ ] Update `schema.graphqls`; hand the regenerated SDL back so the frontend
      copies it into `workforce-frontend/schema.graphql` and adds matching
      `graphql_client` query files under `queries/analytics/`.

## Frontend consumption (for cross-reference)

Each query will get a `queries/analytics/*.graphql` document and a
`graphql_client` struct + fetch fn in `src/graphql/`, called from the analytics
handler, which aggregates into JSON for an ECharts template (the Phase 5
pattern). The frontend does **not** call GraphQL client-side.

Mapping:
- `capabilityGrowth` → Phase H (`/{lang}/analytics/growth`)
- `capabilitySupplyDemand` → Phase I (same page)
- `teamCapabilityMatrix` → Phase C heatmap
- `Team`/`OrgTier` rollups → Phase D org-chart overlay
- `talentMovements` (or frontend reconstruction) → Phase G mobility sankey
