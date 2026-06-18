pub mod clock;
pub mod id;

pub use clock::{Clock, RealClock, TestClock};
pub use id::{IdGenerator, RealIdGenerator, TestIdGenerator};

use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProductId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Product {
    pub id: ProductId,
    pub title: String,
    pub handle: String,
    pub price_cents: u32,
    pub inventory_quantity: u32,
    pub published: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreateProductParams {
    pub title: String,
    pub handle: String,
    pub price_cents: u32,
    pub inventory_quantity: u32,
    pub published: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("Product title is required.")]
    EmptyTitle,
    #[error("Product handle is required.")]
    EmptyHandle,
    #[error("Another product already uses this handle.")]
    DuplicateHandle { handle: String },
}

pub struct Catalog {
    products: Vec<Product>,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
}

impl Catalog {
    pub fn new(clock: Arc<dyn Clock>, id_generator: Arc<dyn IdGenerator>) -> Self {
        Self {
            products: Vec::new(),
            clock,
            id_generator,
        }
    }

    pub fn create_product(&mut self, params: CreateProductParams) -> Result<Product, CatalogError> {
        if params.title.trim().is_empty() {
            tracing::warn!(error_code = "validation_failed", "Product creation validation failed: empty title");
            return Err(CatalogError::EmptyTitle);
        }
        if params.handle.trim().is_empty() {
            tracing::warn!(error_code = "validation_failed", "Product creation validation failed: empty handle");
            return Err(CatalogError::EmptyHandle);
        }
        if self.products.iter().any(|p| p.handle == params.handle) {
            tracing::warn!(
                error_code = "duplicate_product_handle",
                handle = %params.handle,
                "Product creation validation failed: duplicate handle"
            );
            return Err(CatalogError::DuplicateHandle {
                handle: params.handle,
            });
        }

        let id = ProductId(self.id_generator.generate_id());
        let now = self.clock.now();
        let product = Product {
            id,
            title: params.title,
            handle: params.handle,
            price_cents: params.price_cents,
            inventory_quantity: params.inventory_quantity,
            published: params.published,
            created_at: now,
            updated_at: now,
        };
        tracing::info!(
            product_id = %product.id.0,
            product_handle = %product.handle,
            "Product created successfully"
        );
        self.products.push(product.clone());
        Ok(product)
    }

    pub fn list_products(&self) -> Vec<Product> {
        self.products.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_create_and_list_products() {
        let clock = Arc::new(RealClock);
        let id_generator = Arc::new(RealIdGenerator);
        let mut catalog = Catalog::new(clock, id_generator);

        let input1 = CreateProductParams {
            title: "Super Cool T-Shirt".to_string(),
            handle: "super-cool-t-shirt".to_string(),
            price_cents: 2999,
            inventory_quantity: 50,
            published: true,
        };

        let input2 = CreateProductParams {
            title: "Cozy Hoodie".to_string(),
            handle: "cozy-hoodie".to_string(),
            price_cents: 4999,
            inventory_quantity: 20,
            published: false,
        };

        let prod1 = catalog.create_product(input1).unwrap();
        let prod2 = catalog.create_product(input2).unwrap();

        // Verify that fields survived creation for product 1
        assert_eq!(prod1.title, "Super Cool T-Shirt");
        assert_eq!(prod1.handle, "super-cool-t-shirt");
        assert_eq!(prod1.price_cents, 2999);
        assert_eq!(prod1.inventory_quantity, 50);
        assert_eq!(prod1.published, true);

        // Verify UUIDv7 format for IDs (standard uuid format is 36 chars)
        assert_eq!(prod1.id.0.len(), 36);
        assert_eq!(prod2.id.0.len(), 36);

        // Verify that fields survived creation for product 2
        assert_eq!(prod2.title, "Cozy Hoodie");
        assert_eq!(prod2.handle, "cozy-hoodie");
        assert_eq!(prod2.price_cents, 4999);
        assert_eq!(prod2.inventory_quantity, 20);
        assert_eq!(prod2.published, false);

        // Verify catalog list contains both products
        let products = catalog.list_products();
        assert_eq!(products.len(), 2);
        assert_eq!(products[0], prod1);
        assert_eq!(products[1], prod2);
    }

    #[test]
    fn test_deterministic_creation() {
        let fixed_time = chrono::Utc.with_ymd_and_hms(2026, 6, 17, 12, 0, 0).unwrap();
        let clock = Arc::new(TestClock::new(fixed_time));
        let id_generator = Arc::new(TestIdGenerator::new(vec![
            "prod-id-1".to_string(),
            "prod-id-2".to_string(),
        ]));

        let mut catalog = Catalog::new(clock, id_generator);

        let input = CreateProductParams {
            title: "Fixed Product".to_string(),
            handle: "fixed-product".to_string(),
            price_cents: 1000,
            inventory_quantity: 5,
            published: true,
        };

        let prod = catalog.create_product(input).unwrap();

        assert_eq!(prod.id.0, "prod-id-1");
        assert_eq!(prod.created_at, fixed_time);
        assert_eq!(prod.updated_at, fixed_time);
    }
}
