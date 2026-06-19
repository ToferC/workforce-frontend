# Web Starter Rust

This repo is a starter app for my Web-dev. I've probably built something similar about six times, so hopefully this forestalls a 7th.

- [x] Actix-Web w/ async
- [x] Tera for templates
- [x] Diesel accessing Postgresql DB
- [x] User models
- [x] Automated Admin Generation
- [x] Authentication and sign-in
- [x] Email verification and reset password
- [x] Static files
- [x] Fluent integration for i18n

This is the web UI for the People Data Analytics platform. It is a server-side
Actix-Web app (Tera templates, Fluent i18n) that talks to the
`workforce_analytics` GraphQL API. It has **no database of its own** and sends
no email — older starter-template notes about Diesel/SendGrid no longer apply.

## Dependencies

- Rust (stable; 2024 edition)
- A running `workforce_analytics` GraphQL API to point at

## Environment variables

These are the only variables the code reads. Copy `.env.example` to `.env` and
fill in values (loaded at startup via `dotenv`).

| Variable | Required | Format / example | Purpose |
|---|---|---|---|
| `ENVIRONMENT` | No (default `test`) | `production` \| `test` | `production` binds `HOST:PORT`, targets `GRAPHQL_API_TARGET`, and sets **Secure** session cookies (serve over HTTPS). Otherwise binds `127.0.0.1:8088` and targets `http://127.0.0.1:8080/graphql`. |
| `HOST` | If `production` | `0.0.0.0` | Bind address. |
| `PORT` | If `production` | `8088` | Bind port. |
| `GRAPHQL_API_TARGET` | If `production` | `https://api.example.com/graphql` | Full URL of the GraphQL endpoint (include `/graphql`). |
| `COOKIE_SECRET_KEY` | Yes | ≥ **64 bytes** random (`openssl rand -hex 64`) | Keys signed/encrypted session cookies. **Must be ≥64 bytes** — the process panics otherwise. |

> Note: the schema this client generates against (`schema.graphql`) must stay in
> sync with `schema.graphqls` in the `workforce_analytics` repo.

## Local setup

```bash
cp .env.example .env     # set COOKIE_SECRET_KEY (>= 64 bytes)
cargo run                # dev mode: serves http://127.0.0.1:8088, API at :8080
```

In dev mode (`ENVIRONMENT` unset/`test`) it binds `127.0.0.1:8088` and targets
the API at `http://127.0.0.1:8080/graphql`, so run the API locally first.

## Deployment (Docker)

Static assets are embedded into the binary at build time; the runtime image
carries the binary plus `templates/` and `i18n/`.

```bash
cp .env.example .env     # set COOKIE_SECRET_KEY; GRAPHQL_API_TARGET is set in compose
docker compose up --build
```

Or build/run the image directly:

```bash
docker build -t workforce-frontend .
docker run --env-file .env \
  -e ENVIRONMENT=production -e HOST=0.0.0.0 -e PORT=8088 \
  -e GRAPHQL_API_TARGET=https://api.example.com/graphql \
  -p 8088:8088 workforce-frontend
```

Because `production` sets Secure cookies, serve the app behind an
HTTPS-terminating reverse proxy in real deployments (otherwise browsers drop the
session cookie over plain HTTP).
