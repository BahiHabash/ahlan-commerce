--! get_product_by_handle : (id, title, handle, price_cents, inventory_quantity, published, description?, published_at?, created_at, updated_at)
SELECT
    id,
    title,
    handle,
    price_cents,
    inventory_quantity,
    published,
    description,
    published_at,
    created_at,
    updated_at
FROM products
WHERE handle = :handle
LIMIT 1;
