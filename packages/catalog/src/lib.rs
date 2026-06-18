pub mod clock;
pub mod id;

pub use clock::{Clock, RealClock, TestClock};
pub use id::{IdGenerator, RealIdGenerator, TestIdGenerator};

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use sqlx::{PgPool, Row};

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
    pub description: Option<String>,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
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
    pub description: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("Product title is required.")]
    EmptyTitle,
    #[error("Product handle is required.")]
    EmptyHandle,
    #[error("Another product already uses this handle.")]
    DuplicateHandle { handle: String },
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

#[derive(Clone)]
pub struct Catalog {
    pool: PgPool,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
}

impl Catalog {
    pub fn new(pool: PgPool, clock: Arc<dyn Clock>, id_generator: Arc<dyn IdGenerator>) -> Self {
        Self {
            pool,
            clock,
            id_generator,
        }
    }

    pub async fn create_product(&self, params: CreateProductParams) -> Result<Product, CatalogError> {
        if params.title.trim().is_empty() {
            tracing::warn!(error_code = "validation_failed", "Product creation validation failed: empty title");
            return Err(CatalogError::EmptyTitle);
        }
        if params.handle.trim().is_empty() {
            tracing::warn!(error_code = "validation_failed", "Product creation validation failed: empty handle");
            return Err(CatalogError::EmptyHandle);
        }

        let id = ProductId(self.id_generator.generate_id());
        let now = self.clock.now();
        let published_at = if params.published { Some(now) } else { None };

        let row = sqlx::query(
            "INSERT INTO products (id, title, handle, price_cents, inventory_quantity, published, description, published_at, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
             RETURNING id, title, handle, price_cents, inventory_quantity, published, description, published_at, created_at, updated_at"
        )
        .bind(&id.0)
        .bind(&params.title)
        .bind(&params.handle)
        .bind(params.price_cents as i32)
        .bind(params.inventory_quantity as i32)
        .bind(params.published)
        .bind(&params.description)
        .bind(published_at)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| {
            if let Some(db_err) = err.as_database_error() {
                if db_err.code() == Some(std::borrow::Cow::Borrowed("23505")) {
                    tracing::warn!(
                        error_code = "duplicate_product_handle",
                        handle = %params.handle,
                        "Product creation validation failed: duplicate handle"
                    );
                    return CatalogError::DuplicateHandle {
                        handle: params.handle.clone(),
                    };
                }
            }
            CatalogError::Database(err)
        })?;

        let product = Product {
            id: ProductId(row.try_get("id")?),
            title: row.try_get("title")?,
            handle: row.try_get("handle")?,
            price_cents: row.try_get::<i32, _>("price_cents")? as u32,
            inventory_quantity: row.try_get::<i32, _>("inventory_quantity")? as u32,
            published: row.try_get("published")?,
            description: row.try_get("description")?,
            published_at: row.try_get("published_at")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        };

        tracing::info!(
            product_id = %product.id.0,
            product_handle = %product.handle,
            "Product created successfully"
        );
        Ok(product)
    }

    pub async fn list_products(&self) -> Result<Vec<Product>, CatalogError> {
        let rows = sqlx::query(
            "SELECT id, title, handle, price_cents, inventory_quantity, published, description, published_at, created_at, updated_at FROM products"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut products = Vec::new();
        for row in rows {
            products.push(Product {
                id: ProductId(row.try_get("id")?),
                title: row.try_get("title")?,
                handle: row.try_get("handle")?,
                price_cents: row.try_get::<i32, _>("price_cents")? as u32,
                inventory_quantity: row.try_get::<i32, _>("inventory_quantity")? as u32,
                published: row.try_get("published")?,
                description: row.try_get("description")?,
                published_at: row.try_get("published_at")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            });
        }

        Ok(products)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    async fn get_test_pool() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres@localhost:5432/ahlan_commerce".to_string());
        PgPool::connect(&database_url).await.unwrap()
    }

    #[tokio::test]
    async fn test_create_and_list_products() {
        let pool = get_test_pool().await;
        // Clean up before test
        sqlx::query("DELETE FROM products WHERE handle IN ('super-cool-t-shirt', 'cozy-hoodie')")
            .execute(&pool)
            .await
            .unwrap();

        let clock = Arc::new(RealClock);
        let id_generator = Arc::new(RealIdGenerator);
        let catalog = Catalog::new(pool, clock, id_generator);

        let input1 = CreateProductParams {
            title: "Super Cool T-Shirt".to_string(),
            handle: "super-cool-t-shirt".to_string(),
            price_cents: 2999,
            inventory_quantity: 50,
            published: true,
            description: Some("A very cool t-shirt".to_string()),
        };

        let input2 = CreateProductParams {
            title: "Cozy Hoodie".to_string(),
            handle: "cozy-hoodie".to_string(),
            price_cents: 4999,
            inventory_quantity: 20,
            published: false,
            description: None,
        };

        let prod1 = catalog.create_product(input1).await.unwrap();
        let prod2 = catalog.create_product(input2).await.unwrap();

        // Verify that fields survived creation for product 1
        assert_eq!(prod1.title, "Super Cool T-Shirt");
        assert_eq!(prod1.handle, "super-cool-t-shirt");
        assert_eq!(prod1.price_cents, 2999);
        assert_eq!(prod1.inventory_quantity, 50);
        assert_eq!(prod1.published, true);
        assert_eq!(prod1.description, Some("A very cool t-shirt".to_string()));
        assert!(prod1.published_at.is_some());

        // Verify UUIDv7 format for IDs (standard uuid format is 36 chars)
        assert_eq!(prod1.id.0.len(), 36);
        assert_eq!(prod2.id.0.len(), 36);

        // Verify that fields survived creation for product 2
        assert_eq!(prod2.title, "Cozy Hoodie");
        assert_eq!(prod2.handle, "cozy-hoodie");
        assert_eq!(prod2.price_cents, 4999);
        assert_eq!(prod2.inventory_quantity, 20);
        assert_eq!(prod2.published, false);
        assert_eq!(prod2.description, None);
        assert!(prod2.published_at.is_none());

        // Verify catalog list contains both products
        let products = catalog.list_products().await.unwrap();
        assert!(products.contains(&prod1));
        assert!(products.contains(&prod2));
    }

    #[tokio::test]
    async fn test_deterministic_creation() {
        let pool = get_test_pool().await;
        sqlx::query("DELETE FROM products WHERE handle = 'fixed-product'")
            .execute(&pool)
            .await
            .unwrap();

        let fixed_time = chrono::Utc.with_ymd_and_hms(2026, 6, 17, 12, 0, 0).unwrap();
        let clock = Arc::new(TestClock::new(fixed_time));
        let id_generator = Arc::new(TestIdGenerator::new(vec![
            "prod-id-1".to_string(),
            "prod-id-2".to_string(),
        ]));

        let catalog = Catalog::new(pool, clock, id_generator);

        let input = CreateProductParams {
            title: "Fixed Product".to_string(),
            handle: "fixed-product".to_string(),
            price_cents: 1000,
            inventory_quantity: 5,
            published: true,
            description: Some("Deterministic description".to_string()),
        };

        let prod = catalog.create_product(input).await.unwrap();

        assert_eq!(prod.id.0, "prod-id-1");
        assert_eq!(prod.created_at, fixed_time);
        assert_eq!(prod.updated_at, fixed_time);
        assert_eq!(prod.description, Some("Deterministic description".to_string()));
        assert_eq!(prod.published_at, Some(fixed_time));
    }
}
