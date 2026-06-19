--! list_published_products : (id, title, handle, price_cents, inventory_quantity, published, description?, published_at?, created_at, updated_at)
SELECT * FROM products WHERE published = true ORDER BY created_at DESC;
