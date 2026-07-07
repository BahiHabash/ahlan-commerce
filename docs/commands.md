# Commands List

The repository uses `make` to automate common development workflows. Below is a reference for the available targets.

## Running the App

- `make start`: Starts the local development environment (API server, Admin UI, etc.) using `mprocs`.
- `make run-api`: Starts the Axum backend on port 3000.
- `make run-admin`: Starts the frontend Admin dashboard in development mode.

## Database

- `make db-start`: Ensures the local PostgreSQL server is running.
- `make db-create`: Creates the `ahlan_commerce` database if it doesn't already exist.
- `make db-check`: Runs `db-start` and `db-create` sequentially.
- `make db-migrate`: Applies Atlas database migrations.
- `make cornucopia-generate`: Generates type-safe Rust code from SQL queries in `db/queries`.

## Testing & Checks

- `make test`: Runs unit and integration tests across the workspace.
- `make health`: Verifies the API is up by calling the health endpoint (`http://localhost:3000/health`).

## Documentation

- `make docs-api`: Generates the OpenAPI schema (`docs/generated/openapi.json`) and GraphQL schema (`docs/generated/schema.graphql`). This is useful when you add or modify endpoints and need to update the API contract.
- `make docs-api-check`: Runs the generation and ensures there is no drift by checking for git diffs. This command is run in CI to prevent stale documentation from being merged.
