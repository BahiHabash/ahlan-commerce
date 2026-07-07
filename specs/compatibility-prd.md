# Product Requirements Document: Compatibility Adapter

## Overview

We need to add a compatibility adapter for importing an external product payload into our native product creation behavior. This will allow external systems to create products in Ahlan Commerce without transforming the payload manually on the client side.

## User Story

As a third-party integrator or migrating user, I want to send an external product payload to the system so that I can create a product in the native Ahlan Commerce system without having to write custom transformation logic.

## Acceptance Criteria

- **Endpoint**: The system provides a dedicated import endpoint (e.g., `/api/products/import`).
- **Translation**: The external payload fields (e.g., legacy JSON shapes, decimal prices) are correctly mapped to native product fields (`title`, `handle`, `price_cents`, `inventory_quantity`, `published`, `description`).
- **Validation**: Validation errors specific to the external payload format are caught and returned clearly to the client.
- **Native Logic Integrity**: The native catalog logic (e.g., unique handle checks, required fields) still applies and returns appropriate error codes when violated.

## Out of Scope

- Bulk import of multiple products at once.
- Continuous synchronization or webhooks with the external system.
- Mapping arbitrary formats; we will target a single, specific external schema.
- Automatic creation of related entities (like categories or users) that are not present in the native product schema.

## Edge Cases

- **Missing Optional Fields**: The external payload might omit optional fields; the adapter must gracefully map them to `null` or defaults.
- **String Limits & Sanitization**: Strings that exceed native limits or handles with unsupported characters must be rejected or sanitized appropriately.

## Test Cases

- **TC1: Successful Import**: Submitting a fully populated external payload creates the product successfully with a 201 Created status and correct field mapping.
- **TC2: Missing Optional Fields**: Submitting a payload with missing optional fields creates the product successfully with default/null values.
- **TC3: Missing Required Fields**: Submitting a payload missing a required field (e.g., title or handle equivalent) returns a 400 Bad Request with a clear validation error.
- **TC4: Price Conversion**: Submitting a payload with a decimal price (e.g., `35.50`) correctly converts and stores it as `3550` price_cents.
- **TC5: Duplicate Handle**: Submitting a payload with a handle that already exists returns a 409 Conflict using the native catalog validation.
- **TC6: Malformed Payload**: Submitting invalid JSON returns a 400 Bad Request.
