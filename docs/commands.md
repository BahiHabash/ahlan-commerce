# Project Commands

This document lists the available commands defined in the project's [Makefile](file:///e:/2-Projects/MasteryIT_Intern/onboarding/Makefile) for development, testing, and operations.

## Available Targets

### `make start`
Opens the local multi-process development board with `mprocs`.
* **Wraps**: `mprocs -c mprocs.yaml`

### `make stop`
Prints a note explaining that PostgreSQL is managed by your local service manager.

### `make run-api`
Starts the Axum API server in development mode.
* **Wraps**: `cargo run -p api`

### `make test`
Runs all unit and integration tests sequentially across the cargo workspace.
* **Wraps**: `cargo test -- --test-threads=1`

### `make db-start`
Starts local PostgreSQL with the Windows service when available, falls back to `pg_ctl`, then checks that local PostgreSQL is accepting connections.
* **Wraps**: `Start-Service postgresql-x64-16`, `pg_ctl -D ... start`, and `pg_isready -h localhost -p 5432 -U postgres`

### `make db-create`
Creates the local `ahlan_commerce` database if it does not already exist.
* **Wraps**: `psql` and `createdb`

### `make db-check`
Runs the local database readiness and creation steps.
* **Wraps**: `make db-start` and `make db-create`

### `make db-migrate`
Applies all pending database migrations to the local database environment using Atlas.
* **Wraps**: `./atlas.exe migrate apply --env local` (on Windows) or `atlas migrate apply --env local` (on Unix/macOS)

### `make health`
Performs a health check request against the API server to verify that it is running and healthy.
* **Wraps**: `curl -f http://localhost:3000/health`
