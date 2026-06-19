# Local Runtime

Use `make start` to open the local process board.

The process board is powered by `mprocs` and currently includes the local API and a PostgreSQL readiness check:

- `api`: runs `make run-api`, which starts the Axum API with `cargo run -p api`.
- `postgres`: runs `make db-check`, which verifies local PostgreSQL is accepting connections and creates the `ahlan_commerce` database if it is missing.

## Local PostgreSQL

This project uses your local PostgreSQL installation. Start PostgreSQL with your local service manager before running the project.

The default connection settings are:

```sh
DB_HOST=localhost
DB_PORT=5432
DB_USER=postgres
DB_NAME=ahlan_commerce
PG_SERVICE=postgresql-x64-16
PGDATA=E:/Set_up_Porgrams/PostgreSql/data
```

On Windows, `make db-start` first tries the local PostgreSQL service. If Windows does not allow service control, it falls back to `pg_ctl` with `PGDATA`. Override `PGDATA` if your PostgreSQL data directory is different.

Run the local database setup directly with:

```sh
make db-check
make db-migrate
```

## Logs

API logs are in the `api` pane inside the `mprocs` view.

PostgreSQL logs are managed by your local PostgreSQL installation.

## Stopping The Local Stack

Exit the `mprocs` view to stop the foreground API process.

`make stop` does not stop PostgreSQL because the project uses your local PostgreSQL service:

```sh
make stop
```

## Why mprocs Is Local Only

`mprocs` is a local development workflow tool. It gives engineers one visible board for multiple local process logs so request flow is easier to debug across the API and database.

It is not production orchestration. Production service topology, restart policy, networking, secrets, and deployment order belong in deployment configuration and deployment docs.
