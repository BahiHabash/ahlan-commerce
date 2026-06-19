--! create_product(id, title, handle, price_cents, inventory_quantity, published, description, published_at, created_at, updated_at)
INSERT INTO products (id, title, handle, price_cents, inventory_quantity, published, description, published_at, created_at, updated_at)
VALUES (:id, :title, :handle, :price_cents, :inventory_quantity, :published, :description, :published_at, :created_at, :updated_at)
RETURNING *;
