--! list_published_products
SELECT * FROM products WHERE published = true ORDER BY created_at DESC;
