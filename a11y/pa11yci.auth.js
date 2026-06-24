// Authenticated pa11y-ci config for the Workforce frontend.
//
// Unlike the JSON config, a JS module lets us read credentials from the
// environment (ADMIN_EMAIL / ADMIN_PASSWORD from the app's .env) instead of
// committing secrets. Run with:
//
//   ADMIN_EMAIL=... ADMIN_PASSWORD=... npm run a11y
//
// Requires the dev server running on http://127.0.0.1:8088 with seeded data.

const BASE = process.env.A11Y_BASE_URL || 'http://127.0.0.1:8088';
const EMAIL = process.env.ADMIN_EMAIL || '';
const PASSWORD = process.env.ADMIN_PASSWORD || '';

// Each pa11y URL runs in a fresh browser context, so every authenticated page
// logs in first via these actions.
const loginActions = [
  `navigate to ${BASE}/en/log_in`,
  `set field input[name=email] to ${EMAIL}`,
  `set field input[name=password] to ${PASSWORD}`,
  'click element #loginForm button[type=submit]',
  'wait for path to not be /en/log_in',
];

const authedPaths = [
  '/en',
  '/en/organizations',
  '/en/people',
  '/en/roles',
  '/en/teams',
  '/en/skills',
  '/en/tasks',
  '/en/work',
  '/en/products',
  '/en/publications',
  '/en/vacancies',
  '/en/organization/new',
  '/en/person/new',
  '/en/analytics',
  '/en/analytics/coverage',
  '/en/analytics/delivery',
  '/en/analytics/mobility',
  '/en/analytics/growth',
  '/en/analytics/supply-demand',
];

module.exports = {
  defaults: {
    standard: 'WCAG2AA',
    runners: ['axe', 'htmlcs'],
    timeout: 40000,
    wait: 600,
    chromeLaunchConfig: { args: ['--no-sandbox', '--disable-setuid-sandbox'] },
    // The theme toggle is an interactive control still under redesign; exclude
    // it so it doesn't mask real findings during the migration. Remove once the
    // GCDS header lands (Phase 1).
    hideElements: '#theme-toggle',
  },
  urls: authedPaths.map((path) => ({
    url: `${BASE}${path}`,
    actions: loginActions,
  })),
};
