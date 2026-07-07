# API Documentation

Our API documentation comes in two forms: generated and written.

## Generated API Docs

Generated docs explain what the API exposes *now*. These docs are automatically built from our Rust implementation and are checked in CI.

- **REST API**: See the OpenAPI JSON output at `generated/openapi.json`. You can also run `make run-api` and browse to `http://localhost:3000/docs/scalar` to view the interactive Scalar UI.
- **GraphQL API**: The raw GraphQL schema is exported to `generated/schema.graphql`.

To regenerate these files locally:
```bash
make docs-api
```

To verify they are up to date (used in CI):
```bash
make docs-api-check
```

## Written API Docs

Written docs explain *why* the system is shaped this way, covering tradeoffs, setup instructions, and operations that aren't visible purely from an API contract.

- [Setup Guide](setup.md)
- [Architecture](architecture.md)
- [Commands List](commands.md)
- [Operations Manual](operations.md)
