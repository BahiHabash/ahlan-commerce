# Product PRD

## User Story
As a merchant, I want to create and list products so that I can manage my store's catalog and display it to customers.

## Acceptance Criteria

### 1. Valid Create
When a merchant provides valid product details (title, handle, price, inventory quantity), the system should create the product and store it persistently. The product should be accessible in subsequent list operations.

### 2. Duplicate Handle
Each product must have a unique handle. If a merchant attempts to create a product using a handle that already exists in the system, the request must be rejected with an appropriate error indicating the conflict.

### 3. List Empty Products
When the catalog has no products, listing products should return an empty list rather than an error.

### 4. List Persisted Products
When the catalog has products, listing products should return all the previously created products with their correct details, ensuring data is persisted correctly.

### 5. Invalid Input
If a merchant attempts to create a product with missing required fields (like an empty title or an empty handle), the system must reject the request with a validation error detailing which fields are invalid.
