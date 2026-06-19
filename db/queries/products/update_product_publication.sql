--! update_product_publication(published, published_at, updated_at, id)
UPDATE products
SET published = :published, published_at = :published_at, updated_at = :updated_at
WHERE id = :id
RETURNING *;
