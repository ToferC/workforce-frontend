# Phase 3 — Forms → GCDS components (implementation notes)

**Plan:** [GC Design System & accessibility compliance plan](../GC_DESIGN_SYSTEM_COMPLIANCE.md)
**Scope:** form rendering + accessibility. Submission contract, CSRF, server
validation, and HTMX flows are deliberately unchanged.

## Why the swap is submission-safe

The GCDS form components (`gcds-input`, `gcds-textarea`, `gcds-select`) use shadow
DOM **but are form-associated custom elements** (`formAssociated: true`, verified
in the package source). They report their value to the enclosing `<form>` under
their `name` via `ElementInternals.setFormValue()`, so:

- **Plain POST + redirect** submits the same `name=value` pairs as before.
- **HTMX** serializes via `FormData(form)`, which includes form-associated
  elements — unchanged.
- `gcds-date-input` would submit the same `YYYY-MM-DD`, but it renders as three
  separate fields; to avoid any value-format risk the **native `<input
  type="date">` is kept**.

## What changed

### `templates/macros/forms.html`
- `text_input` → **`<gcds-input>`**, `textarea` → **`<gcds-textarea>`**,
  `select` → **`<gcds-select>`** (options still passed as slotted `<option>`s).
  **Macro signatures are unchanged**, so no caller/template had to change for the
  field swap. All `type=` values in use (email/number/url/text) are valid for
  `gcds-input`.
- Each field macro gained an optional **`error`** parameter → renders the
  field's associated `error-message` (WCAG 3.3.1). `help` maps to the GCDS
  `hint`.
- Placeholders are dropped (no caller used them; GCDS treats placeholders as an
  a11y antipattern — labels + hints replace them).
- New **`error_summary(heading="")`** macro → `<gcds-error-summary listen="true">`.
  Placed at the top of a form, on submit it aggregates the gcds-* field
  validation errors, takes focus, and links to each offending field
  (WCAG 2.4.3 / 3.3.1).
- **Kept native:** `date_input` (accessible already; preserves submitted value
  format), `checkbox` (single boolean — `gcds-checkboxes` is a group component),
  `csrf`, and the submit button (reliability).

### Form templates (23 files)
`{{ forms::error_summary() }}` was inserted immediately after
`{{ forms::csrf(token=csrf_token) }}` in every form that uses a validated field
macro (text_input/textarea/select). Files with two forms (e.g.
`account/profile.html`, `role/requirement_form.html`) get one summary per form.
The insertion is purely additive — it renders nothing unless a field reports an
error. Retire/confirmation and the hand-rolled auth forms were intentionally
skipped (no validated GCDS fields).

## Compatibility checks performed
- Macro syntax: 8 `macro`/`endmacro` pairs, balanced `{% %}` and `{{ }}`.
- The military/civilian mutual-exclusivity script (`base.html`) reads
  `[name=…].value` and sets `.disabled` — both work on the form-associated
  `gcds-input`/`gcds-select` hosts (they expose `value`/`disabled` props).
- `index.html` search select already has a real `<label for="levelSelect">`, so
  the plan's "placeholder-as-label" concern doesn't apply; that custom-JS search
  form is left untouched.

## Not verified here (needs a running dev server)
- I could not run the app (no DB/API), so submission, HTMX swaps, and the error
  summary focus behaviour are **not runtime-tested**. The reasoning above is
  from the component source, not observation.
- **Server-driven per-field errors:** the macros now *accept* an `error`
  string, but handlers still surface validation failures via flash messages
  only. Wiring handlers to pass per-field errors is a handler (business-logic)
  change, intentionally out of this CSS/template-focused phase.

## Manual test checklist (when a dev server is up)
- [ ] Create + edit each entity: fields submit and persist as before.
- [ ] Submit a create form with a required field empty → field shows an inline
      error and the error summary lists/links it and takes focus.
- [ ] HTMX org-chart add-role/add-team inline forms still submit and swap.
- [ ] Military vs civilian fields still disable each other.
- [ ] Keyboard-only completion of a representative form; `npm run a11y` shows
      no new violations.
