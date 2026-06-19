# syntax=docker/dockerfile:1
#
# Workforce frontend (Actix-Web + Tera + GraphQL client). Multi-stage build of
# the `frontend` binary onto a slim Debian runtime.
#
# Static assets are embedded into the binary at build time (build.rs +
# actix-web-static-files), so only the binary, Tera templates, and Fluent i18n
# files are needed at runtime.

# ---- Build stage ------------------------------------------------------------
FROM rust:latest AS builder

# OpenSSL for reqwest's TLS (default-tls). pkg-config to locate it.
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Whole crate: build.rs bundles ./static, the graphql_client macros read
# ./queries + schema.graphql, and Fluent reads ./i18n — all at build time.
COPY . .

RUN cargo build --release

# ---- Runtime stage ----------------------------------------------------------
FROM debian:bookworm-slim

# CA roots + OpenSSL for outbound HTTPS to the GraphQL API.
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -r app && useradd --no-log-init -r -g app app

WORKDIR /app

# Binary plus the assets it reads at runtime (relative to this working dir):
#   - Tera templates : "templates/**/*"
#   - Fluent locales : "./i18n/"
# (static/ is embedded in the binary, so it is not copied.)
COPY --from=builder /app/target/release/frontend ./frontend
COPY templates templates
COPY i18n i18n

USER app

# Default port; the app binds HOST:PORT in production (see .env.example).
EXPOSE 8088

CMD ["./frontend"]
