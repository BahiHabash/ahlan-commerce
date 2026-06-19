# Ahlan Commerce

> A guided onboarding book for fresh new-commer who know Rust basics. A micro app that simulate daily tasks and decisions during the development process.

## Documentations

- [Learning Log](./docs/learning-log.md)
- [Local Runtime](./docs/local-runtime.md)
- [Project Commands](./docs/commands.md)

## Run Locally

This project runs the API against a local PostgreSQL installation. Docker is not required for the default local workflow.

### Requirements

- Rust and Cargo
- GNU Make
- PostgreSQL client tools on `PATH`: `psql`, `createdb`, `pg_isready`, and `pg_ctl`
- Local PostgreSQL data directory configured by `PGDATA`
- `atlas.exe` in the project root on Windows, or `atlas` on `PATH` on Unix/macOS
- `cornucopia` if you need to regenerate database query code

On Windows, if `make` is installed through GnuWin32 and the current terminal does not see it yet, refresh the current PowerShell session:

```powershell
$env:Path += ';C:\Program Files (x86)\GnuWin32\bin'
```

### Safe Startup

From the project root:

```powershell
make db-check
make db-migrate
make run-api
```

`make db-check` verifies PostgreSQL is accepting connections and creates the `ahlan_commerce` database if it is missing. It does not drop or reset data.

The default database settings are:

```text
DB_HOST=localhost
DB_PORT=5432
DB_USER=postgres
DB_NAME=ahlan_commerce
PG_SERVICE=postgresql-x64-16
PGDATA=E:/Set_up_Porgrams/PostgreSql/data
```

Override any value inline when your local setup differs:

```powershell
make db-check DB_USER=my_user PGDATA=E:/PostgreSQL/data
```

### Verify

With the API running, open another terminal:

```powershell
make health
```

Expected response:

```json
{"status":"ok"}
```

The API listens on `http://localhost:3000` by default.
