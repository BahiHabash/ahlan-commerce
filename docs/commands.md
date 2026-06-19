# Project Commands

This document lists the available commands defined in the project's [Makefile](file:///e:/2-Projects/MasteryIT_Intern/onboarding/Makefile) for development, testing, and operations.

## Available Targets

### `make start`
Opens the local multi-process development board with `mprocs`.
* **Wraps**: `mprocs -c mprocs.yaml`

### `make stop`
Stops the local PostgreSQL container.
* **Wraps**: `docker stop catalog-db`

### `make run-api`
Starts the Axum API server in development mode.
* **Wraps**: `cargo run -p api`

### `make test`
Runs all unit and integration tests sequentially across the cargo workspace.
* **Wraps**: `cargo test -- --test-threads=1`

### `make db-start`
Starts the local PostgreSQL container or spins up a new instance if it doesn't exist.
* **Wraps**: `docker start catalog-db || docker run --name catalog-db -p 5432:5432 -e POSTGRES_HOST_AUTH_METHOD=trust -d postgres`

### `make db-logs`
Follows the local PostgreSQL container logs.
* **Wraps**: `docker logs -f catalog-db`

### `make db-migrate`
Applies all pending database migrations to the local database environment using Atlas.
* **Wraps**: `./atlas.exe migrate apply --env local` (on Windows) or `atlas migrate apply --env local` (on Unix/macOS)

### `make health`
Performs a health check request against the API server to verify that it is running and healthy.
* **Wraps**: `curl -f http://localhost:3000/health`
