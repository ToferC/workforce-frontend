# Products / Tasks / Work — delivery roadmap

This roadmap came out of a gap review of the **Product → Task → Work** domain
from two perspectives (the project officer/manager who plans and tracks
delivery, and the employee who does the work). The review found the hierarchy
could be *created* but not *operated*: no dates or lifecycle signals on work, no
history, no way to raise or triage issues, no approval gate, no real
dependencies, and no check that priority was applied consistently down the
tiers.

The fixes shipped in three tiers. Every slice is a paired change — the API
(`workforce_analytics`) and the frontend (`workforce-frontend`) — with the GraphQL
SDL (`schema.graphqls` ↔ `schema.graphql`) kept in sync.

## Status — all tiers complete

| Tier | Proposal | What it delivers | Status |
|------|----------|------------------|--------|
| 1 | Dates & actionable BLOCKED | `dueDate` / `startedAt` / `completedAt`, a structured blocked reason (free text + optional blocking role), and overdue/blocked indicators. | ✅ Merged |
| 2 | Status history (P4) | `work_status_history` records every status transition, rendered as a timeline on the work page. | ✅ Merged |
| 2 | My Work (P5) | A per-person queue of the work assigned to the signed-in user. | ✅ Merged |
| 2 | Comments & flags (P3) | A comment/flag stream on work; anyone occupying the role (or a manager) can comment, with a manager flags queue for triage. | ✅ Merged |
| 3 | Approval workflow (P7b) | Tasks carry an approval state (draft → pending → approved/rejected) with submit/approve/reject mutations and an approver queue. | ✅ Merged |
| 3 | Work dependencies (P7a) | "Blocked by" as a real work-to-work relationship with cycle prevention, an `isBlocked` signal, and "can't start yet" indicators. | ✅ Merged |
| 3 | Priority consistency (P7c) | Flags where priority "drops" between a product, its tasks, and their work, via a review page and inline badges. | ✅ Merged |

## Where each capability lives

- **Work detail page** (`/{lang}/work/{id}`): status history timeline, comment/flag
  stream, dates + overdue/blocked indicators, a Dependencies card (blocked-by /
  blocks, with cycle-safe add), and a "priority below task" flag.
- **Task detail page**: approval state and controls, plus "priority below product"
  and lower-priority-work flags.
- **Product detail page**: a priority-mismatch count linking to the review.
- **My Work** (`/{lang}/work/mine`): the signed-in user's assigned work.
- **Flags queue** and **Approvals queue**: manager/approver triage lists, each
  scoped to the area the caller manages.
- **Priority Consistency** (`/{lang}/analytics/consistency`): every task whose
  priority is out of step with the tier above it, highest priority first.

## Conventions used throughout

- Access is enforced server-side on the API (`user < analyst < operator < admin`)
  and mirrored in the frontend handlers; template role checks only hide controls.
- Mutating POSTs validate a CSRF token; user feedback goes through flash messages.
- Priority is ordered `LOW < MEDIUM < HIGH < CRITICAL`; a "mismatch" is a child
  ranked below its parent.
- All user-facing strings are bilingual (EN/FR) via Fluent.
