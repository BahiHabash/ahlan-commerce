# Manual Browser Check

Playwright E2E tests have been implemented in `apps/admin/tests/product.spec.ts`. However, if the automated tests are blocked in your environment (e.g., due to missing browser dependencies or CI restrictions), you can perform the browser check manually.

## Mentor Verification Steps

1. Start the complete stack using `make start` or `mprocs`.
2. Open a browser and navigate to the admin UI: `http://localhost:5173/products`.
3. Verify that the empty state or the list of products is visible.
4. Fill out the **Create Product** form with the following details:
   - **Title**: `Manual Test Product`
   - **Handle**: `manual-test-product`
   - **Price (cents)**: `1500`
   - **Inventory**: `10`
   - **Description**: `Testing manual creation.`
   - **Published**: Checked
5. Click **Create Product**.
6. Observe the success message.
7. Verify that the new product immediately appears in the table on the left **without** a full page refresh.
