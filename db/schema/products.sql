CREATE TABLE products (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    handle TEXT NOT NULL UNIQUE,
    price_cents INTEGER NOT NULL,
    inventory_quantity INTEGER NOT NULL,
    published BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    description TEXT,
    published_at TIMESTAMPTZ
);
