# Manual Schema Change and Atlas Migration Record

This document records the manual changes made to the database schema for Chapter 04, Task 04.2, and our transition to declarative migrations with Atlas in Task 04.3 and 04.4.

---

## Part 1: Reflections on Manual Schema Changes

### Applied SQL Changes
The following SQL queries were executed manually in the local database `ahlan_commerce` to introduce product descriptions and publishing timestamps:
```sql
ALTER TABLE products ADD COLUMN description TEXT;
ALTER TABLE products ADD COLUMN published_at TIMESTAMPTZ;
```

### 1. What did you have to remember?
When modifying the schema manually, we had to remember:
* The exact syntax for adding columns in PostgreSQL.
* To match the DB data types with the corresponding Rust compiler representation (`description TEXT` -> `Option<String>`, `published_at TIMESTAMPTZ` -> `Option<chrono::DateTime<chrono::Utc>>`).
* To make the new columns nullable so existing rows in production wouldn't crash because of constraint violations.
* To update all query strings (both `INSERT` and `SELECT`) across the entire repository to read and write the new fields.
* To update the DTO request and response mapping layers (`dto.rs`) so the endpoint serializes/deserializes fields correctly.

### 2. What broke when Rust code and DB schema disagreed?
Before running the schema changes:
* If the Rust code expects the columns to be there (e.g. `SELECT description, published_at`) but they do not exist in the database, SQLx queries will fail at runtime with a database error saying "column does not exist".
* If we write to the database using query binds that include the new columns but the database doesn't have them, the insert query fails at runtime.
* In general, any mismatch between Rust structs, SQL queries, and the database schema causes immediate database driver errors, returning `500 Internal Server Error` to the api consumers.

### 3. Why would this be dangerous in production?
* **Lack of Single Source of Truth**: There is no version control of the database schema. It is easy to lose track of what columns exist in which environments (e.g. Dev vs. Staging vs. Production).
* **High Risk of Downtime**: If the app code is deployed before the manual schema changes are run (or vice-versa), the server will fail to execute database queries and crash or fail requests.
* **Human Error**: Forgetting to run a command, running the wrong command, or executing it against the wrong database (or typo-ing table names) can cause irreversible data loss or prolonged outages.
* **Non-repeatable**: Manual changes cannot be easily automated in CI/CD pipelines, making deployments fragile and slow.

---

## Part 2: Transition to Atlas Migrations

Declarative schema management using Atlas solves these manual migration discipline issues.

### Atlas Commands

To manage migrations locally, we run:

1. **Generate Migration Files**: Compare our declarative schema file `db/schema/products.sql` with our local development/dev database to calculate differences and output a new migration file:
   ```bash
   ./atlas.exe migrate diff <migration_name> --env local
   ```

2. **Apply Migrations**: Apply all pending migration files from `db/migrations/` sequentially to the target database:
   ```bash
   ./atlas.exe migrate apply --env local
   ```

3. **Check Migration Status**: Check if the target database is up-to-date with migration files:
   ```bash
   ./atlas.exe migrate status --env local
   ```

---

## Part 3: What Atlas Solves vs. What It Doesn't

### What Atlas Solves
* **Schema Change Discipline**: It defines a single source of truth for the schema (`db/schema/products.sql`).
* **Repeatability**: Anyone can run `atlas migrate apply` to instantly bring their local or remote database to the exact target state.
* **Reviewability**: Schema changes are output as migration files that can be checked into Git, code-reviewed, and automatically tested in CI/CD.

### What Atlas Does NOT Solve (Rust Query Safety & Code Drift)
Atlas does **not** solve the mismatch between application query code and the database schema:
* **No Compile-time Query Safety**: We are still writing queries as plain string literals in Rust (e.g. `sqlx::query("SELECT ...")`). If we rename a column or table in our schema file and run Atlas, the Rust compiler remains unaware.
* **Code-Drift and Query-Drift**: When the schema changes, we must manually search the codebase for raw SQL strings and update them, leaving us vulnerable to runtime query panics. To solve this, we would need query code generation tools (like SQLx offline mode, sqlx-macro compile-time checking, or Cornucopia) that link compiler checks directly to schema definitions.
