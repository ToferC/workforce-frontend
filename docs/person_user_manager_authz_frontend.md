# Frontend Track — Person-as-User & Manager-Scoped Authz

Frontend (Rust/Tera/HTMX) side of the
[backend roadmap](https://github.com/toferc/workforce_analytics/blob/claude/lucid-fermat-4hird7/docs/roadmap_person_user_manager_authz.md).
The backend coordinates the staged workflow in
`workforce_analytics/docs/workflow_person_user_manager_authz.md`.

> **This track is GATED.** It unblocks once the API publishes the Phase 2.3
> contract (`me { … managedTierIds }`, `redeemInvite`, access-denied error
> semantics) on `workforce_analytics` `claude/vigilant-feynman-ow83pu`. Until
> then this file is the tracking stub so the track is visible and reviewable.

## Contract this track builds against (from backend Phase 2.3)

- `me` query → caller identity + span of control:
  ```graphql
  query Me {
    me {
      user { id email role }
      person { id givenName familyName }
      managedTierIds
      isManager
    }
  }
  ```
- `redeemInvite(accessKey: String!, password: String!)` → sets a new user's
  password from the invite token.
- Person creation no longer accepts `userId` (auto-provisioned by the API).
- Mutations outside the caller's span return an access-denied GraphQL error
  (same channel as today's `RoleGuard` denials).

> `schema.graphql` here must stay in sync with `schema.graphqls` in
> `workforce_analytics` once the contract lands (see `CLAUDE.md`).

## Tasks (adapted from roadmap FE-1…FE-6 to Tera/HTMX conventions)

- [ ] **FE-1** Add `queries/me.graphql` + `MeQuery` wrapper in `src/graphql/`;
      thread `managed_tier_ids` / `is_manager` through `generate_basic_context`
      so every template can gate on span. Calls go through `post_graphql`.
- [ ] **FE-2** Span-aware gating helper `can_manage(tier_id)` exposed to Tera;
      hide Edit / Assign Task / Assign Work / Reparent / Change-owner controls
      for entities outside `managed_tier_ids`. (Template checks hide buttons
      only — the API enforces.)
- [ ] **FE-3** Drop the `userId` field from the person create/edit forms and the
      `create_person.graphql` mutation variables.
- [ ] **FE-4** Invite activation page + CSRF-validated POST handler for
      `redeemInvite`, keyed off the `access_key` token.
- [ ] **FE-5** Pre-filter product-owner / task / work assignment pickers to the
      manager's scope.
- [ ] **FE-6** Surface access-denied API errors as flash messages
      (`security::add_flash(session, "danger", …)`) rather than 500s.

## Conventions to respect (see `CLAUDE.md`)

- All API calls go through `post_graphql` in `src/graphql/client.rs` (sends
  `Authorization: Bearer <token>`, returns `ApiError`). No hand-rolled reqwest.
- Mutating POST handlers validate `csrf_token` via `security::verify_csrf_token`
  and enforce access with `security::require_role(...)`.
- Forms use `templates/macros/forms.html`; progressive enhancement via vendored
  HTMX, but forms must still work as plain POST + redirect.
