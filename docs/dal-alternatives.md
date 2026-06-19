# DAL Alternatives

When building a Data Access Layer in Rust, there are several popular approaches. Here is a comparison of ORMs, checked queries, and SQL-first code generation.

| Approach | Pros | Cons | Protects Against | Does Not Protect Against |
| :--- | :--- | :--- | :--- | :--- |
| **Diesel** (ORM) | - High-level model for common CRUD.<br>- Mature ecosystem.<br>- Compile-time type checking. | - Heavy macro usage.<br>- Can obscure underlying SQL.<br>- Complex queries fight the abstraction. | - SQL syntax errors.<br>- Type mismatches. | - Hiding expensive query behavior.<br>- Abstraction leaks on complex queries. |
| **SeaORM** (ORM) | - Async-first.<br>- Dynamic queries.<br>- Built on `sqlx`. | - SQL behavior less visible.<br>- Overhead of mapping models.<br>- Engineers still need to know DB behavior. | - SQL syntax errors.<br>- Basic relationships. | - N+1 query problems hidden behind abstractions. |
| **SQLx** (Checked Queries) | - Keeps SQL visible.<br>- Compile-time or offline checks catch type mismatches. | - Queries are still embedded in Rust code.<br>- Does not create a clear DAL boundary by itself. | - Type mismatches.<br>- Syntax errors. | - Sprawling data access logic across handlers.<br>- Queries scattered in domain logic. |
| **Cornucopia** (SQL-First) | - SQL stays in named `.sql` files.<br>- Rust gets generated typed query functions.<br>- Reviewers can inspect SQL easily. | - Requires regeneration step when query files change.<br>- Needs manual DAL boundary wrapping. | - Type mismatches.<br>- SQL syntax errors.<br>- Hiding query complexity. | - Architecture decay (handlers must be disciplined to use DAL, not generated code directly). |

## Why Ahlan Uses SQL-First Generated Code With Cornucopia

We use SQL-first code generation (Cornucopia) because the SQL remains visible and reviewable as raw SQL in dedicated files, while Rust still benefits from strictly typed query bindings. This fits our design philosophy better than hiding query behavior behind an ORM. However, we still wrap the generated code inside our own custom DAL (`catalog` package) because the generated functions are mere persistence primitives, and we need an application boundary to enforce policy.

## Why SQL-First Code Generation Still Needs Discipline

SQL-first generation doesn't automatically create a good architecture. Engineers must remain disciplined by:
1. Not letting handlers call the generated database queries directly.
2. Wrapping the generated bindings in a well-defined Data Access Layer (DAL) that dictates how queries are orchestrated.
