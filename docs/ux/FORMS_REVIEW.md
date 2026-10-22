# UX Review: Data Input and Forms

Scope: every create/edit form in `templates/` (25 forms), the shared macros in
`templates/macros/forms.html`, and the POST handlers in `src/handlers/`, as of
`a148780`. Companion to [`UX_FLOW_REVIEW.md`](UX_FLOW_REVIEW.md), which covers
the read-only pages.

Method:
- Read each form template and its handler.
- Ran the app locally against the seeded demo data (2,151 active roles, 391
  tiers) and drove every form in Chromium with Playwright, in English and
  French and at phone width. Each run submitted empty, invalid, duplicate and
  stale-token input.

Every finding below was reproduced in the browser unless it is marked
*(code)*.

## 1. What already works

- **Main entity forms keep input on API errors.** Organization, tier, team,
  role, person, product, task, work and publication re-render with the typed
  values intact.
- **Client-side required checks are good.** The GCDS error summary lists every
  empty required field, links to it and moves focus to it, in both languages.
- **Large-choice pickers are in place where they were built:** the two-step
  domain → skill picker, and the person typeahead on role creation.
- **Enum options use the shared labels.** Selects show "People & Leadership",
  not `PEOPLE_AND_ORGANISATIONAL_LEADERSHIP`.

## 2. Findings, highest impact first

### P1 — the form fails or the user's input is lost

1. **Required dropdowns look filled but submit nothing.**
   - On a create form, a required `forms::select` with no `blank_label` shows
     its first option (Domain "Combat", Level "Desired", Language "English").
     But it has no `value`, so it submits an empty value.
   - Pressing Save then reports "Choose an option to continue" on a field that
     visibly has one, and the user must re-pick the same option.
   - Affected: tier, team, product, task and skill (domain); requirement,
     capability, validation and work (level); language.
   - **Fix:** in `forms::select`, always render a disabled prompt option
     when the select is required and nothing is selected. That already happens
     when a `blank_label` is passed, so default `blank_label` to a generic
     "— Select —" key.
2. **Contract form crashes when Description is left empty.**
   - An empty `gcds-*` field is omitted from the submission entirely; it is
     not sent as an empty string.
   - `ContractForm.description` has no `#[serde(default)]`, so actix shows a
     bare text page: `Parse error: missing field 'description'.` It has no
     layout or way back, and everything typed is lost.
   - Separately, the app has no `web::FormConfig` error handler, so *any*
     malformed post produces that same raw page *(code)*.
   - **Fix:**
     - Add `#[serde(default)]` to `ContractForm.description`.
     - Add a `FormConfig` error handler that flashes a friendly message and
       redirects back to the referring form.
     - Add a test that posts each form with every optional field omitted.
3. **Validation errors on secondary forms throw the input away.**
   - The contract form checks "The end date cannot precede the start date",
     then *redirects* to an empty form.
   - The same redirect-on-error pattern is used by the contract (create/edit),
     pay-rate, budget, language, affiliation, requirement, capability,
     validation, my-capability and profile handlers *(code)*.
   - **Fix:** re-render the form with the submitted values (the
     `*_from_form` pattern the entity forms already use).
4. **A stale or invalid CSRF token discards the form.**
   - Every handler redirects to a blank form with "Invalid form token".
   - Sessions expire, so a long form typed slowly is lost.
   - **Fix:** on a CSRF failure, re-render the posted values with a fresh
     token. Do not persist anything; the user still has to press Save again.
5. **API and database errors are shown verbatim, in English only.**
   - A duplicate tier name shows
     `duplicate key value violates unique constraint "org_tiers_org_name_en_key"`,
     on the French page too.
   - `ApiError` passes the message straight through.
   - **Fix:** map known API errors to Fluent keys, for example
     `error-duplicate-name`. Attach each one to its field (see 6) and keep the
     raw text out of the UI; log it instead.
6. **Fractional role effort is impossible.**
   - `Role.effort` is a float, and its help text says "full time is around 10".
   - But the field is `type="number"` with no `step`, so 0.5 or 2.5 is
     rejected: "The two nearest valid values are 0 and 1".
   - **Fix:** add a `step` parameter to `forms::text_input` and pass
     `step="0.5"` (or `any`).

### P2 — friction and avoidable errors

7. **No server error is ever shown next to its field.**
   - Every input macro accepts `error=`, but no template passes one.
   - All server feedback is a single banner at the top, and the GCDS error
     summary stays empty for server-side failures.
   - **Fix:** return a field → message map from handler validation and API-error
     mapping, and pass it into `error=`. The summary then lists them
     automatically.
8. **Dropdowns too long to scan.**
   - Product owner and work "blocked on" list **2,152 roles**.
   - Parent tier and team tier list **391 tiers**.
   - **Fix:** reuse the `person_picker` pattern, as a typeahead backed by a
     `role_options?q=` / `tier_options?q=` endpoint. Alternatively, scope the
     choices to the organization or team.
9. **Tier level is typed by hand and can contradict the parent.**
   - "Create tier" under a level-2 parent defaults the level to **1**.
   - **Fix:** derive the level from the parent (parent + 1, or 1 with no
     parent) and show it read-only.
10. **Fields required that the data doesn't need.**
    - Organization: website URL, both acronyms, and a free-text "type".
    - Person: phone, street address, city, province, postal code and country.
      Province and country are also free text.
    - Task: URL.
    - Each one blocks saving a real record that lacks it.
    - **Fix:**
      - Make these optional (check the API's input types first).
      - Turn organization type, province and country into selects.
11. **Edit forms can't change what create set.**
    - "Edit role" only offers titles, active and dates. A role's
      classification, rank, occupational group and effort can't be corrected
      without recreating the position.
    - "Edit team" can't show the current domain, because the API's `Team` type
      has no `primaryDomain`. Its select therefore says "keep current".
    - **Fix:**
      - Add the missing fields to the role edit form.
      - Add `Team.primaryDomain` to the API.
12. **Hard-coded English label.** "Priority" is a literal string on the task,
    work and product forms, so French users see English.
13. **No double-submit guard.** Save stays enabled while the request runs, so a
    double click on create can make duplicate records *(code)*. Add
    `hx-disabled-elt="find button[type=submit]"` and a plain-JS fallback.

### P3 — consistency and polish

14. **Two field styles on one form.**
    - The GCDS inputs are large and marked "(required)".
    - The native date inputs, person picker, skill picker, radios and checkbox
      are small, Bootstrap-styled and marked with a red `*`.
    - The role, contract, work and capability forms mix both.
    - **Fix:** use `<gcds-date-input>` for dates, and style the remaining native
      controls with the same tokens and the same "(required)" text.
15. **Save is smaller than Cancel.** The primary button renders visibly smaller
    than the secondary one on every form, so the visual weight is inverted.
    Give both the same size and keep Cancel low-emphasis, as a link or
    secondary GCDS button.
16. **Person form address row overlaps at desktop width.** "Province
    (required)" runs into "Postal code (required)", and the postal-code and
    country boxes are cramped. Give the row two columns (city/province,
    postal code/country).
17. **Forms don't say where you are.**
    - Entity forms show only Home › Section.
    - Nine sub-forms show no breadcrumb at all: capability, validation,
      language, affiliation, requirement, contract, team owner, role edit and
      user.
    - The contract form doesn't name its task.
    - **Fix:** reuse `page::trail` with the parent record, and add a subtitle
      naming it ("Contract for *Task title*").
18. **"Edit Role Status" has no error summary**, unlike every other form.

## 3. Suggested sequencing

| Phase | Scope | Findings |
|---|---|---|
| 1. Stop losing input | `forms::select` prompt default, `ContractForm` default and a `FormConfig` handler, re-render on validation and CSRF errors, `step` on effort, the "Priority" label | 1–4, 6, 12 |
| 2. Errors people can act on | API error → Fluent key + field map, wired into `error=`, double-submit guard | 5, 7, 13 |
| 3. Fewer, better fields | Derived tier level, optional or select fields, typeahead role and tier pickers, complete role edit (API: `Team.primaryDomain`) | 8–11 |
| 4. One visual language | GCDS date input, button sizing, address layout, form breadcrumbs, role-edit error summary | 14–18 |

Phase 1 is mostly in the macros and a handful of handlers, and it fixes the
failures users hit today. Each phase should add a render or POST test that
pins its behaviour, like the existing `tests/templates_render.rs` cases.
