# Implementation Plan: Fully Enabling Capabilities & Work

## Goal

Turn Workforce into a fully functioning HR application that supports matching
**people and their capabilities** against **work and its requirements**, with
supporting analytics. The user-facing target:

- On a **person**, add/manage capabilities. *(already works — see below)*
- On **work**, set its requirements (skill/domain/level) and **assign a
  person**, ideally from a ranked list of qualified candidates.
- Managers can see **vacancies**, **capability gaps**, and **capacity** at a
  glance.

## Key finding: the API is ahead of the frontend

A review of `schema.graphql` shows the backend **already exposes everything**
needed. The work is almost entirely frontend (handlers + queries + templates).
No API/schema changes are required for Phases 1–4.

Relevant API surface already available:

| Capability | API surface |
| --- | --- |
| Assign / reassign work to a role (person) | `WorkData.roleId`, `WorkData.taskId`, `WorkData.skillId` on `updateWork`; `NewWork.roleId` (optional) on `createWork` |
| Work-specific skill requirement | `Work.skill` / `NewWork.skillId` / `WorkData.skillId` |
| Find qualified people for a piece of work | `Work.capabilityMatches(count): [Capability!]!` (matches on skill if set, else domain, ordered by validated level) |
| Find people for a vacant role | `Role.findMatches: [Person!]!` *(already shown on role page)* |
| Find roles for a person | `Person.findMatches: [Role!]!` *(already shown on person page)* |
| Vacant work | `Product.vacantWork`, `Task.vacantWork` |
| Vacant roles | `Query.vacantRoles(count)`, `Team.vacantRoles`, `OrgTier.vacantRoles` |
| Capacity / load | `Person.activeEffort`, `Role.effort`, `Task.effort` |
| Requirements | `createRequirement`, `updateRequirement` (update unused in UI) |

## Current state assessment

### Already implemented (no work needed)
- **Capabilities on a person** — `src/handlers/capability.rs`: add
  (`create_capability_*`), retire (`retire_capability_post`), validate
  (`validate_capability_*`). Surfaced on `templates/person/person.html`.
- **Skills** — full CRUD (`src/handlers/skill.rs`).
- **Products** — full CRUD (`src/handlers/product.rs`).
- **Tasks** — full CRUD (`src/handlers/task.rs`), created under a role.
- **Roles** — create (assign person by typed name), edit (active/dates).
  `Role.findMatches` (people matching role requirements) shown on role page.
- **Requirements (on a role)** — create + retire (`src/handlers/role.rs`).
- **Work** — create (under a role), read, edit (limited fields only).

### Gaps to close
1. **Work cannot be assigned / reassigned from the UI.** `work_form.html` only
   edits description/url/domain/level/effort/status. There are no `task_id`,
   `role_id`, or `skill_id` fields, even though `updateWork` accepts all three.
   `work.html` shows "Not assigned" with no action to assign.
2. **No way to create or manage vacant work.** `work.rs:136` notes vacant work
   "is created elsewhere" — but nowhere actually does.
3. **`Work.capabilityMatches` is never queried.** There is no "qualified
   people for this work" list and no one-click assign.
4. **No work index / dashboard.** No `allWork` query or work listing; vacant
   and unassigned work is undiscoverable.
5. **Requirements cannot be edited** (only created/retired) — `updateRequirement`
   is unused.
6. **No vacancy dashboards** despite `vacantWork` / `vacantRoles` existing.
7. **No analytics** (capability coverage, requirement gaps, capacity).

---

## Phased implementation

### Phase 1 — Assign & reassign work (core of the request)

Make work assignable to a role (and therefore to a person), and let work carry
a specific skill.

- **GraphQL**
  - Extend `queries/work/update_work.graphql` to send `roleId`, `taskId`,
    `skillId`.
  - Add `queries/work/all_work.graphql` (used in Phase 4) — optional here.
  - Confirm `WorkData` codegen includes the new optional fields.
- **Handlers** (`src/handlers/work.rs`)
  - Add `role_id`, `skill_id` (optional), and allow `task_id` to the
    `WorkForm` and `edit_work_post`, passing them through to `update_work`.
    (`role_id` empty string → `None` = unassign / make vacant.)
  - Add `assign_work_form` / `assign_work_post`
    (`GET|POST /{lang}/work/{work_id}/assign`) — a focused "assign to role"
    action that lists candidate roles (reuse `role_options` from
    `product.rs`).
  - Add a "create vacant work" entry point: allow `create_work_*` to be reached
    from a **task** (`/{lang}/task/{task_id}/work/new`) with no role, in
    addition to the existing role-scoped route.
- **Templates**
  - `work_form.html`: add `role` (assignee) select, optional `skill` select,
    and on edit a `task` select. Keep plain POST + redirect (HTMX optional).
  - `work.html`: in the "Assigned To" card, add an **Assign / Reassign**
    button (operator+); when unassigned, show **Assign** prominently.
- **Routes** (`src/handlers/routes.rs`): register the new services *before*
  the `{work_id}` catch-all, matching the existing ordering convention.
- **Security/CSRF**: mirror existing handlers —
  `require_role(MinimumRole::Operator)` + `verify_csrf_token`.

### Phase 2 — Capability matching surfaced on work

Let an operator pick the best person for a piece of work.

- **GraphQL**: extend `queries/work/work_by_id.graphql` to request
  `capabilityMatches { id validatedLevel selfIdentifiedLevel person { id givenName familyName activeEffort } skill { nameEn } }`.
- **Templates** (`work.html`): add a **"Qualified people"** card listing
  matches with validated level and current effort, each with a one-click
  **Assign** button (POSTs to the Phase 1 assign endpoint with that person's
  role). Show effort so over-loaded people are visible.
- Optional: a small badge on `work.html` flagging when the assigned person's
  validated level is **below** the work's required `capabilityLevel`
  (gap warning).

### Phase 3 — Requirement editing + work requirement clarity

- **Handlers** (`src/handlers/role.rs`): add `edit_requirement_form` /
  `edit_requirement_post` using the existing `updateRequirement` mutation
  (`queries/requirements/update_requirement.graphql` already exists).
- **Templates** (`role/role.html`): add an **Edit** button per requirement.
- **Docs/UX note**: a *role* has `Requirement`s (skill + required level); a
  *work* item carries its own requirement inline (`domain` +
  `capabilityLevel` + optional `skill`). Phase 1 makes that work-level
  requirement fully editable, satisfying "add requirements on work."

### Phase 4 — Discovery: work index & vacancy dashboards

- **Work index** — `GET /{lang}/work`, new `queries/work/all_work.graphql`,
  `templates/work/work_index.html`. Columns: description, task/product, domain,
  level, effort, status, assignee (or **Vacant** badge). HTMX search/filter
  like `person_index.html`; filter by status and "unassigned only".
- **Vacancy dashboard** — `GET /{lang}/vacancies`: combine
  `Query.vacantRoles(count)` and per-product/task `vacantWork`. Each row links
  to its assign action (Phase 1) and shows candidate counts via
  `findMatches` / `capabilityMatches`.
- **Nav**: add Work, Vacancies (and existing Products/Tasks/Skills) links to
  the top nav (`templates/base.html` / partial) for discoverability.

### Phase 5 — Analytics

Reporting on top of the now-connected data (analyst+ read access).

- **Capability coverage** — per `SkillDomain` / skill: count of people by
  validated level; identify thin areas. Built from `allPeople`/`skills`
  capability data.
- **Capability gap analysis** — required levels (role `Requirement`s + work
  `capabilityLevel`) vs. available validated capabilities, highlighting
  shortfalls per domain/team.
- **Capacity & utilization** — `Person.activeEffort` and `Role.effort`
  aggregated per team/org; flag over-allocated (>10) and under-utilized people.
- **Work/vacancy rollups** — counts by `WorkStatus`, vacant work and vacant
  roles by team/product, aging against `targetCompletionDate`.
- **Delivery** — `GET /{lang}/analytics` dashboard with cards/tables; consider
  Chart.js for visuals. If aggregation is heavy, propose dedicated analytics
  resolvers in the API repo (only place a schema change might be warranted).

---

## Cross-cutting conventions (follow existing patterns)
- All API calls via `post_graphql` (`src/graphql/client.rs`); add a wrapper fn
  per new query in the matching `src/graphql/*.rs` module.
- Guard mutating handlers with `security::require_role(...)` and
  `security::verify_csrf_token(...)`; queue feedback with `security::add_flash`.
- Register new routes before catch-all `{id}` routes in `routes.rs`.
- Forms use `templates/macros/forms.html`; all strings bilingual via Fluent.
- Keep `schema.graphql` in sync with the API's `schema.graphqls` if Phase 5
  introduces new resolvers.

## Suggested sequencing & scope
1. **Phase 1** — highest value, unblocks the core "assign a person to work"
   request. (Handlers + work form/view + routes.)
2. **Phase 2** — makes assignment smart (matching). Small, high impact.
3. **Phase 3** — requirement editing parity.
4. **Phase 4** — discovery (work index + vacancies).
5. **Phase 5** — analytics layer.

Phases 1–2 alone deliver the headline workflow: *open a work item → see ranked
qualified people → assign one.* Each phase is independently shippable.
