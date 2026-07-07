# Architecture Overview

This document explains the high-level architecture of the Ahlan Commerce system.

## Workspace Structure

The repository uses a Cargo workspace with multiple packages:
- **`apps/api`**: The main HTTP server entrypoint. Built with Axum, it exposes REST endpoints and a GraphQL endpoint.
- **`packages/catalog`**: Contains the core business logic and domain entities (e.g. Products).
- **`packages/db`**: Handles database connectivity and raw SQL queries using Cornucopia and `tokio-postgres`.

## API Layer

We expose two APIs for consumers:
1. **REST API**: Built with Axum handlers. Provides standard resource manipulation. Interactive documentation is available via Scalar (`/docs/scalar`).
2. **GraphQL API**: Built with `async-graphql`. Accessible at `/graphql`. Provides flexible querying capabilities.

Both APIs use a shared error-handling mechanism and depend on the same underlying `catalog` business logic.

## Data Access Layer (DAL)

We use a Postgres database and rely on a strict separation between business logic and database execution:
- Migrations are managed by **Atlas** using HCL/SQL.
- Type-safe queries are generated using **Cornucopia**, mapping raw SQL to Rust structs without needing an ORM.

## Observability

The API uses `tracing` and `tower-http` to structure logging and attach unique `x-request-id` headers to all requests. This enables robust monitoring and easier debugging in production.
