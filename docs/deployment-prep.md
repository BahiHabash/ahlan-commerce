# Deployment Preparation and Runtime Contract

This document defines the production runtime boundaries and processes required to successfully deploy Ahlan Commerce. 

## Required Runtime Processes

- **API Service:** The core backend application logic (Rust).
- **Worker Service:** The background job processor (Rust).
- **Admin Frontend:** The web application for store management (Node/Next.js/React).
- **PostgreSQL Service:** The primary database.
- **Redis Service:** The caching and task queuing backend.

## Environment Variables

All required environment variables must be explicit. The `API` and `Worker` binaries will intentionally crash at startup if these are missing.

- `API_BIND_ADDR`: The host and port for the API service (e.g., `0.0.0.0:3000`). Required by: **API Service**.
- `DATABASE_URL`: The full PostgreSQL connection string. Required by: **API Service** and **Worker Service**.
- `REDIS_URL`: The Redis connection string. Required by: **API Service**.
- `VITE_ADMIN_PUBLIC_API_URL`: The URL that the Admin Frontend will use to reach the API (e.g., `https://api.example.com`). Required by: **Admin Frontend**.

See `.env.example` for reference templates.

## Build Commands

These commands produce the artifacts ready for production:

- **API and Worker Build:** `make build-release` (runs `cargo build --release`)
- **Admin Build:** `make build-admin` (runs `cd apps/admin && npm ci && npm run build`)

## Start Commands

These commands run the compiled artifacts in production. **Note on Coolify/Docker:** Do not use `make` inside Docker containers as it traps termination signals and prevents graceful shutdowns. Use the raw commands instead.

- **API Start:** 
  - *Bare Metal:* `make start-api`
  - *Coolify / Docker:* `api` (the Dockerfile copies the binary to `/usr/local/bin`)
- **Worker Start:** 
  - *Bare Metal:* `make start-worker`
  - *Coolify / Docker:* `worker` (the Dockerfile copies the binary to `/usr/local/bin`)
- **Admin Start:** 
  - *Bare Metal:* `make start-admin`
  - *Coolify / Nixpacks:* `npm start` (or `npm run start`)

## Migrations

Migrations are managed externally via Atlas, ensuring they are separated from application code startups. 

- **Migration Command:** 
  - *Bare Metal:* `make db-migrate-prod`
  - *Coolify Pre-Deploy Script:* `atlas migrate apply --env production --url $DATABASE_URL`
- Migrations must be run and verified *before* rolling out new instances of the API or Worker services.

## Health Checks

- **API Health Check:** `curl -f http://localhost:3000/health` (Using the configured `API_BIND_ADDR` port)
- **Redis Health Check:** `redis-cli ping`
- **PostgreSQL Health Check:** `pg_isready -h $DB_HOST -U $DB_USER`

## Deployment Principles

1. **Build from committed source only.** No manual patches in production.
2. **Do not bake secrets into images.** All secrets pass through environment variables.
3. **Fail startup loudly on missing env vars.** Fallbacks belong in local development, not in production.
4. **Keep API and worker starts explicit.** Even if built in the same Dockerfile, they remain isolated, independently scalable processes.
