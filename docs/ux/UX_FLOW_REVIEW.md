# UX & Logical-Flow Review: Organization → Tier → Team → Role → Person

Scope: the detail and index pages along the org hierarchy (`templates/organization`,
`org_tier`, `team`, `role`, `person`) as of `aefc6da`. The lens: which facts a
viewer needs in order to understand the unit and decide something, in what order,
and how quickly the page makes those facts readable.

## 1. What each level should answer

| Level | The viewer's question | Decisions it should support |
|---|---|---|
| Organization | How big is it, how is it funded, and where are the gaps? | Which tier to look at next, and where to invest or reallocate |
| Tier | How do my sub-units compare? | Budget roll-down, owner assignment, which team needs attention |
| Team | Who is here, what are they delivering, and are we at capacity? | Fill vacancies, rebalance work, add roles |
| Role | Is it filled, how well, what does it cost, and who does it report to? | Fill, transfer, vacate, end |
| Person | Where do they sit, how loaded are they, and what can they do? | Assign, develop, validate skills |

Right now each page mostly answers *"what records are attached to this one?"*.
The findings below reorder the pages around the questions above.

## 2. Cross-cutting findings (highest impact first)

1. **The hierarchy is invisible.** No detail page renders breadcrumbs. Only the
   `*_form.html` pages use `{% block breadcrumb %}`. So a role never shows its
   Org › Tier › Team chain.
   - The tier page has **no link back to its organization**. Only the org-chart
     buttons reach it.
   - The role page shows only the team, with no tier or org.
   - The person page shows the org, and the team only inside the role cards.
   - **Fix:** add a shared `breadcrumbs` macro and render
     `Org › Tier(s) › Team › Role` on every detail page. The tier chain needs a
     parent walk: `orgTierNode`, or `parentOrganizationTier` recursively.
2. **Facts are duplicated between the meta strip and the sidebar.** #87 added the
   header meta strip without removing the sidebar copies:
   - Org: type and website.
   - Tier: owner, parent, and level. Headcount also appears in the meta strip
     *and* in a tile.
   - Team: owner and tier. The filled/vacant count appears in the meta strip *and*
     in the members card badge.
   - Role: incumbent, team, and salary, each shown twice.
   - Person: email and phone.

   **Fix:** make the meta strip the single home for identity facts. Keep only
   relationship or detail content in the sidebar (owner email, reporting line,
   languages).
3. **KPI tiles are copy-pasted and low-signal.** About 15 hand-rolled
   `card > fs-2 fw-bold` tiles across 4 pages.
   - They lead with numbers that drive no decision: *Capability types* (a count of
     rows), a raw *Total effort* with no denominator, and *Products*.
   - The numbers that do drive decisions are missing: **vacancies**,
     **utilization** (`totalEffort / (headcount × 10)`), **overdue work**, and
     **budget variance**.
   - **Fix:** add a `viz::stat_tile(value, label, tone, href)` macro. Standardize a
     4-tile row per level: Headcount · Vacancies · Utilization · FY variance.
     Make each tile link to the filtered list behind it.
4. **Vacant and Retired look identical.** Both use `badge bg-warning text-dark`.
   `role_list.html` uses `bg-danger` for vacant. **Fix:** add a
   `viz::vacancy_chip` and a distinct neutral `viz::retired_chip`, and use them
   everywhere.
5. **Action overload in headers.** An operator sees up to 7 equal-weight buttons:
   - Tier: Org chart, Visual view, Edit, Add child tier, Assign owner, Retire.
   - Role: Edit, Find candidates, Add task, Add work, Vacate, End.

   Destructive actions sit next to primary ones. **Fix:** one primary action per
   page (Tier: *Add team*; Team: *Add role*; Role: *Fill* when vacant, *Edit*
   when filled). Put everything else in a secondary "More" menu. Move
   Retire/End/Vacate to a "Manage" footer section.
6. **Bilingual leaks in content.** The headings are localized, but the entity
   names in links and lists hard-code `nameEn`, `titleEnglish`, or `nameEnglish`:
   - Top tiers, the tier parent and children, team → tier, team members, the role
     page's team, the person's roles and org, and the people/team/role list
     secondary text.
   - The tier page has hard-coded English: "Capabilities by Domain", "Domain", and
     "Total Capabilities".
   - **Fix:** add a `name(obj)` macro, or resolve the name server-side, and use it
     everywhere.
7. **Every card has the same visual weight.** Every section is `card shadow-lg`
   with an `h2.h4` header plus an icon, so the primary section (members, matches)
   looks no different from the secondary ones (publications, affiliations).
   **Fix:** use `shadow-sm` or bordered cards by default. Reserve emphasis for the
   one decision section per page.
8. **Index pages are bare name lists.** Rows carry no decision data:
   - Organizations: headcount and vacancies.
   - Teams: headcount, vacancies, and owner.
   - Roles: classification and status.
   - People: current role.

   The filters are also inconsistent:
   - Teams have no org or tier filter and no *New team* button.
   - Roles have no ended/retired toggle.
   - Organizations have no pagination.

   **Fix:** use a shared table-style row with 2–3 key columns and the same filter
   bar on every index page.
9. **Raw enums and dates are rendered as-is.** Examples: `primaryDomain`,
   `personnelType`, the `req.domain`/`requiredLevel` text on person job matches,
   and `dueDate | truncate(10)`. **Fix:** use the `viz::*` chips and a
   locale-aware date filter.

## 3. Per-level findings

### Organization (`organization/organization.html`)

- **Order:** finance tiles (shown only if finances exist), then the tier list,
  then a collapsed capability table. With no finances, the page opens on a plain
  list.
  - **Proposed:** Stats row (headcount, vacancies, utilization, budget), then a
    **top-tier comparison table** (owner, headcount, vacancies, FY projected
    vs. allocation), then capability-by-domain chips (same as tier and team),
    then affiliations.
- The top-tier rows show the raw `primaryDomain` and nothing comparable. Add
  `headcount` and `totalEffort` to `organization_by_id.graphql`'s `topOrgTier`,
  since both already exist on `OrgTier`.
- The capability summary is a flat, unsorted `name/domain/level/count` table,
  collapsed by default. The tier and team pages use a domain summary instead.
  Reuse that here, and consider a chart (`charts::chart`).
- `publications` is fetched but never rendered. The sidebar links to *all*
  publications instead. Either render the org's publications or drop the field
  from the query.
- In the affiliations card, each row links back to the *current* org
  ("Org — role"). Show only the person and the affiliation role.
- `organization_by_id` fetches financials **sequentially** per top tier
  (`src/handlers/organization.rs`). Use `join_all`, as the tier page already
  does. That tier page's concurrency fix is #85.

### Tier (`org_tier/org_tier.html`)

- The page has no link to its organization. See §2.1.
- **Order:** tiles, budget (with the inline allocation form), teams, then child
  tiers (hidden when a budget exists, because they are folded into the budget
  table), then capabilities.
  - The structure (child tiers and teams) is what the viewer navigates. Finance is
    one attribute of it.
  - **Proposed:** one **"Sub-units" comparison table** with a row per child tier
    and per team: owner, headcount, vacancies, utilization, FY allocation vs.
    projected. Follow it with the budget summary. Move the *Set allocation* form
    behind an "Edit allocation" disclosure.
- The child tiers inside the budget table lose their owner and headcount, and the
  fallback list has neither. The comparison table fixes both.
- The team rows guard `team.headcount is defined`, but the field is always
  queried. Drop the guard.
- The *Capability types* tile counts rows in `capabilityCounts`. Replace it with
  vacancies.

### Team (`team/team.html`)

- **Order:** 4 tiles plus a second row of 3 finance tiles, i.e. 7 tiles before the
  first person appears. Collapse them into one 4-tile row: Headcount ·
  Vacancies · Utilization · FY lapse.
- **Members table:**
  - It has no column headers.
  - The filled rows read *person, role*, but the vacant rows (in a `card-footer`)
    read *role*. Merge them into one table with columns Role · Holder (or a
    Vacant chip) · Classification · Effort, sorted by reporting line.
    `reportsToId` is already fetched and unused. That lets the lead appear first
    and direct reports indent under them.
  - A member with active effort ≥ 9 should be flagged (overloaded) in the row, not
    just shown as a red bar.
  - *Find candidates* is a duplicate of the title link. Deep-link it to
    `/role/{id}#matches`.
- **Delivery at a glance:**
  - The tasks have no due date or overdue flag. The role and person pages already
    compute `overdue_work_ids`.
  - Active work repeats per work item with no grouping. Group it by task.
- The *Capabilities by domain* card is nested awkwardly inside the sidebar (broken
  indentation). It should also be compared to the team's roles' **requirements**
  (supply vs. demand), which is the decision signal. The analytics
  `supply-demand` view already has this.
- `team.retiredAt != "Still Active"` compares against a sentinel string. Normalize
  it in the handler to `retired: bool`, as the other entities use `retiredAt`
  truthiness.

### Role (`role/role.html`)

- **Primary button:** its label is "Edit role status" (`edit-role-status`) but
  it opens the full edit form. Rename it.
- **Vacant layout:** the matches are in the main column, while the requirements
  and the direct-assign form are in the sidebar footer, so the two ways to fill
  the role are split across columns.
  - **Proposed:** in the main column, show Requirements, then **Fill this role**
    (matcher and direct-assign as two tabs or two stacked options).
- **Filled layout:** Requirements plus the incumbent fit chart come first, which
  is good. Add the incumbent's **start date and tenure**; it is available in the
  `assignments` where `isCurrent`.
- Missing context: tier and org (breadcrumbs), the active/ended status as a meta
  item (currently only a title badge), and the effort of the role vs. its work
  total.
- Reporting line: good content, but it sits under Cost in the sidebar. Move it
  directly under the meta strip, or into the breadcrumb row, because it is the
  most-used navigation from a role.
- **Assignment history:** the dates are raw, and the current holder is repeated
  with the meta strip. Show only past holders, or label the list "History".

### Person (`person/person.html`)

- **Bug:** *Past roles* renders an **empty card** when every assignment is
  current, because the `{% else %}` of a `for` only fires on an empty iterable,
  not when an `if` filters out every item. Filter in the handler, or compute
  `past = assignments | filter(attribute="isCurrent", value=false)`.
- **Bug:** the account status resolves only for admins (`userByEmail` is
  admin-guarded). An **operator** therefore sees *Grant access* on every person,
  including active users. Gate the button on `role == "admin"`, or on a resolved
  status.
- The meta strip is wrapped in `{% if account_status or person.activeEffort is defined %}`,
  so email and phone vanish if neither is present. Drop the outer guard.
- The header subtitle shows only the first active role. A person with 2 roles at
  50% each reads as holding one. Show all of them, or "Role A + 1 more".
- **Order:** active role cards (each repeating the team owner's email), then
  capabilities, then job matches, then past roles.
  - **Proposed:** Current roles (compact: role · team · effort), then
    **Capabilities vs. current role requirements** (gap chips), then the radar,
    then job matches, then history.
  - The radar is under a potentially long list. Put it beside the list, or first.
- **Job matches:**
  - They render requirements as raw text (`name (DOMAIN) — LEVEL`), with no fit
    score and no comparison to this person's levels. The role page has a scored
    matcher (`macros/match.html`). Reuse it so both directions read the same.
  - Consider limiting the matches to operators/admins and the person themself.
- Contact card: it renders `", "` when the address is empty, and it duplicates
  the email and phone from the meta strip.
- `personnelType` is shown as a raw enum badge. Localize it.

## 4. Suggested sequencing

| # | Change | Size |
|---|---|---|
| 1 | Person past-roles empty card and operator *Grant access* bugs; drop the person meta guard | S |
| 2 | Breadcrumb macro on all detail pages, plus the tier → org link | S–M |
| 3 | Remove meta-strip/sidebar duplicates; localize the names (`name()` macro) and the tier's hard-coded English | S |
| 4 | `stat_tile`, `vacancy_chip`, and `retired_chip` macros; standard 4-tile row per level | M |
| 5 | Header action hierarchy (one primary action, a "More" menu, a danger section) | M |
| 6 | Sub-unit comparison table on the org and tier pages (query adds headcount and vacancies) | M |
| 7 | Merged team members table ordered by reporting line; role "Fill this role" section | M |
| 8 | Person capabilities vs. requirements; reuse the scored matcher for job matches | M–L |
| 9 | Index pages: key columns and a consistent filter bar | M |
