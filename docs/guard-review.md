# Guard Skills Review

## clean-code-guard Findings

Finding: The parameter `ext` is an abbreviation. Names should reveal intent (Rule 1).
Decision: accepted
Reason: `ext` is vague. `external_product` is clearer and avoids mental mapping.
Follow-up: Renamed `ext` to `external_product` in `adapter.rs`.

Finding: The function returns `Result<..., String>`. Error handling uses a generic string instead of a typed error (Rule 15).
Decision: accepted
Reason: String errors are an anti-pattern in Rust because they swallow the type of the failure, making it hard for callers to match and handle specific errors.
Follow-up: Created a custom `AdapterError` enum.

Finding: The function does not validate string lengths or sanitize the slug (handle), despite the boundary requirement in the PRD (Rule 16).
Decision: needs mentor review
Reason: We need product input on what the maximum lengths should be, and whether we should truncate overflowing strings or reject the request entirely.
Follow-up: Added to mentor review sync agenda.

Finding: The price conversion uses floating-point math (`price_val * 100.0`), which is notoriously unsafe for monetary calculations.
Decision: needs mentor review
Reason: To fix this properly, we should use a crate like `rust_decimal`, but we need architectural approval before introducing new numeric dependencies.
Follow-up: Added to mentor review sync agenda.

## test-guard Findings

Finding: `test_map_external_product` performs filesystem I/O to read `external-product.json` for a pure function test. (Rule 2/9 - Boundaries)
Decision: accepted
Reason: Testing a pure mapping function does not require disk I/O. Loading files makes the test brittle and slow.
Follow-up: Replaced filesystem I/O with an inline JSON string.

Finding: Test names like `test_map_external_product` echo the function signature instead of describing the scenario. (Rule 5)
Decision: accepted
Reason: Test names should read like requirements.
Follow-up: Renamed tests to `test_valid_payload_maps_correctly` and `test_unparseable_price_returns_error`.

## docs-guard Findings

Finding: The PRD claims "Missing Optional Fields" are handled gracefully (TC2), but the `ExternalProduct` struct does not use `Option<T>` or `#[serde(default)]`. Serde will reject payloads missing any field. (Rule 3)
Decision: accepted
Reason: The code drifts from the PRD requirement. Missing fields will currently cause a 400 Bad Request instead of using defaults.
Follow-up: Added `#[serde(default)]` to `stock` and `is_visible` in `ExternalProduct` to align with the PRD.
