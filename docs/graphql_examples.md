# GraphQL Examples

## Example Query: List Products

This query lists all products, requesting their `id`, `title`, `handle`, and timestamps.

```graphql
query ListProducts {
  products {
    id
    title
    handle
    priceCents
    inventoryQuantity
    published
    description
    createdAt
    updatedAt
    publishedAt
  }
}
```

## Example Mutation: Create Product

This mutation creates a new product and returns the created fields.

```graphql
mutation CreateProduct {
  productCreate(input: {
    title: "Awesome T-Shirt"
    handle: "awesome-t-shirt"
    priceCents: 2500
    inventoryQuantity: 100
    published: true
    description: "A very awesome t-shirt made of cotton."
  }) {
    id
    title
    handle
    priceCents
    inventoryQuantity
    published
    description
    createdAt
    updatedAt
    publishedAt
  }
}
```
