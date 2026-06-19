# Local Runtime

Use `make start` to open the local process board.

The process board is powered by `mprocs` and currently includes only the local API and PostgreSQL:

- `api`: runs `make run-api`, which starts the Axum API with `cargo run -p api`.
- `postgres`: runs `make db-start`, then follows the `catalog-db` container logs with `make db-logs`.

## Logs

API logs are in the `api` pane inside the `mprocs` view.

PostgreSQL logs are in the `postgres` pane inside the `mprocs` view. The pane follows `docker logs -f catalog-db`, so it shows database container output after the container starts.

## Stopping The Local Stack

Exit the `mprocs` view to stop the foreground API process and the foreground database log follower.

Run `make stop` to stop the local PostgreSQL container:

```sh
make stop
```

## Why mprocs Is Local Only

`mprocs` is a local development workflow tool. It gives engineers one visible board for multiple local process logs so request flow is easier to debug across the API and database.

It is not production orchestration. Production service topology, restart policy, networking, secrets, and deployment order belong in deployment configuration and deployment docs.
