# Data Access Layer (DAL)

The `ahlan-commerce` project enforces a strict boundary for database access. Handlers and domain orchestration logic should never execute raw SQL or directly call generated database bindings. Instead, they must call functions exposed by our public-safe Data Access Layer (the `catalog` package).

## Where Things Live

- **Schema & Migrations**: The database schema and its history live in the `db/migrations/` directory.
- **SQL Query Files**: The actual SQL queries are maintained in `db/queries/products/`.
- **DAL Code**: The public Rust interface wrapping persistence logic lives in `packages/catalog/`.

## Atlas vs. Cornucopia

It is crucial to understand the distinction between these two tools:

- **Atlas** is responsible for **database schema migration history**. It ensures that your database's tables, columns, and indexes transition correctly over time.
- **Cornucopia** is responsible for **typed Rust access to named SQL queries**. It takes the `.sql` files in `db/queries/` and generates Rust functions so you can safely call them without worrying about parameter or return type mismatches.

**Atlas does not replace Cornucopia. Cornucopia does not replace Atlas.**

### When to run Atlas
Run Atlas when you need to change the structure of the database itself (e.g., adding a table, adding a column, modifying an index).

### When to run Cornucopia
Run Cornucopia (via `make cornucopia-generate`) when you add, modify, or delete the `.sql` queries that your application uses to interact with the database.

## Why Handlers Do Not Own Persistence
Handlers are responsible for parsing HTTP requests and formatting HTTP responses. If they own persistence, the SQL logic becomes scattered, difficult to test, and tightly coupled to the web framework. By enforcing a DAL boundary, we keep the data access centralized, reviewable, and independently testable.
