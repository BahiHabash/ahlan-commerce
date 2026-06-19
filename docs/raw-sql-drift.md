# Raw SQL Drift

When managing SQL as raw strings embedded directly within Rust application code, the connection between database expectations and application code structure is fragile.

## Which SQL strings had to change?
When adding a new query for published products or updating paths that touched `description` and `published_at`, the inline SQL strings inside the `catalog` library (e.g., `INSERT INTO...`, `SELECT * FROM...`, or new `UPDATE...` strings) had to be carefully modified by hand.

## Which Rust structs had to change?
If a column was added or type changed, the Rust representation (e.g., `Product` struct) needed matching updates. If a query was expected to return only specific fields, we had to either map them back to the existing struct or create a new struct for partial views.

## What could compile while still being wrong?
Rust compilation checks syntax and basic types, but it **cannot** verify the contents of a raw string literal. 
- A typo in a column name (e.g., `descirption` instead of `description`).
- Passing the wrong number of parameters (`$1, $2, $3` but only binding two variables).
- Assuming `inventory_quantity` is an `i32` when the database expects `i64`.
All of these will compile perfectly but fail at runtime.

## What could leak to production if a query was missed?
If you updated the schema via Atlas but forgot to update one of the scattered raw SQL queries, the application would likely crash on that specific endpoint when invoked. If it's a rarely used admin path or edge case, the mismatch could easily slip past testing and leak to production, causing unexpected 500 errors or corrupted data states. Atlas checks the database schema, but it does not catch missed updates to the inline SQL strings in the Rust codebase.
