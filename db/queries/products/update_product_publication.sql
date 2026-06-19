--! update_product_publication(published, published_at?, updated_at, id) : (id, title, handle, price_cents, inventory_quantity, published, description?, published_at?, created_at, updated_at)
UPDATE products
SET published = :published, published_at = :published_at, updated_at = :updated_at
WHERE id = :id
RETURNING *;
