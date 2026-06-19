pub mod clock;
pub mod id;

pub use clock::{Clock, RealClock, TestClock};
pub use id::{IdGenerator, RealIdGenerator, TestIdGenerator};

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use deadpool_postgres::Pool;
use chrono::{DateTime, Utc};

use db::queries::products::{create_product, list_products, list_published_products, update_product_publication};

/// Helper: convert a `DateTime<FixedOffset>` returned by cornucopia to `DateTime<Utc>`.
fn to_utc(dt: DateTime<chrono::FixedOffset>) -> DateTime<Utc> {
    dt.with_timezone(&Utc)
}

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
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input struct for creating a product (used by API layer).
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
    #[error("Product not found.")]
    ProductNotFound { id: String },
    #[error("Database error: {0}")]
    Database(#[from] tokio_postgres::Error),
    #[error("Pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),
}

#[derive(Clone)]
pub struct Catalog {
    pool: Pool,
    clock: Arc<dyn Clock>,
    id_generator: Arc<dyn IdGenerator>,
}

impl Catalog {
    pub fn new(pool: Pool, clock: Arc<dyn Clock>, id_generator: Arc<dyn IdGenerator>) -> Self {
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

        let id = self.id_generator.generate_id();
        let now = self.clock.now();
        let published_at: Option<DateTime<Utc>> = if params.published { Some(now) } else { None };

        // Cornucopia uses DateTime<FixedOffset>; convert from Utc.
        let now_fixed: DateTime<chrono::FixedOffset> = now.with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
        let published_at_fixed: Option<DateTime<chrono::FixedOffset>> =
            published_at.map(|dt| dt.with_timezone(&chrono::FixedOffset::east_opt(0).unwrap()));

        let client = self.pool.get().await?;

        let row = create_product::create_product()
            .bind(
                &client,
                &id.as_str(),
                &params.title.as_str(),
                &params.handle.as_str(),
                &(params.price_cents as i32),
                &(params.inventory_quantity as i32),
                &params.published,
                &params.description.as_deref(),
                &published_at_fixed,
                &now_fixed,
                &now_fixed,
            )
            .one()
            .await
            .map_err(|err| {
                if let Some(db_err) = err.as_db_error() {
                    if db_err.code() == &tokio_postgres::error::SqlState::UNIQUE_VIOLATION {
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
            id: ProductId(row.id),
            title: row.title,
            handle: row.handle,
            price_cents: row.price_cents as u32,
            inventory_quantity: row.inventory_quantity as u32,
            published: row.published,
            description: row.description,
            published_at: row.published_at.map(to_utc),
            created_at: to_utc(row.created_at),
            updated_at: to_utc(row.updated_at),
        };

        tracing::info!(
            product_id = %product.id.0,
            product_handle = %product.handle,
            "Product created successfully"
        );
        Ok(product)
    }

    pub async fn list_products(&self) -> Result<Vec<Product>, CatalogError> {
        let client = self.pool.get().await?;

        let rows = list_products::list_products()
            .bind(&client)
            .all()
            .await?;

        let products = rows
            .into_iter()
            .map(|row| Product {
                id: ProductId(row.id),
                title: row.title,
                handle: row.handle,
                price_cents: row.price_cents as u32,
                inventory_quantity: row.inventory_quantity as u32,
                published: row.published,
                description: row.description,
                published_at: row.published_at.map(to_utc),
                created_at: to_utc(row.created_at),
                updated_at: to_utc(row.updated_at),
            })
            .collect();

        Ok(products)
    }

    pub async fn list_published_products(&self) -> Result<Vec<Product>, CatalogError> {
        let client = self.pool.get().await?;

        let rows = list_published_products::list_published_products()
            .bind(&client)
            .all()
            .await?;

        let products = rows
            .into_iter()
            .map(|row| Product {
                id: ProductId(row.id),
                title: row.title,
                handle: row.handle,
                price_cents: row.price_cents as u32,
                inventory_quantity: row.inventory_quantity as u32,
                published: row.published,
                description: row.description,
                published_at: row.published_at.map(to_utc),
                created_at: to_utc(row.created_at),
                updated_at: to_utc(row.updated_at),
            })
            .collect();

        Ok(products)
    }

    pub async fn update_product_publication(
        &self,
        id: &str,
        published: bool,
    ) -> Result<Product, CatalogError> {
        let now = self.clock.now();
        let published_at: Option<DateTime<Utc>> = if published { Some(now) } else { None };

        let now_fixed: DateTime<chrono::FixedOffset> =
            now.with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
        let published_at_fixed: Option<DateTime<chrono::FixedOffset>> =
            published_at.map(|dt| dt.with_timezone(&chrono::FixedOffset::east_opt(0).unwrap()));

        let client = self.pool.get().await?;

        let row = update_product_publication::update_product_publication()
            .bind(
                &client,
                &published,
                &published_at_fixed,
                &now_fixed,
                &id,
            )
            .opt()
            .await?
            .ok_or_else(|| {
                tracing::warn!(product_id = %id, "Product not found for publication update");
                CatalogError::ProductNotFound { id: id.to_string() }
            })?;

        let product = Product {
            id: ProductId(row.id),
            title: row.title,
            handle: row.handle,
            price_cents: row.price_cents as u32,
            inventory_quantity: row.inventory_quantity as u32,
            published: row.published,
            description: row.description,
            published_at: row.published_at.map(to_utc),
            created_at: to_utc(row.created_at),
            updated_at: to_utc(row.updated_at),
        };

        tracing::info!(
            product_id = %product.id.0,
            published = %product.published,
            "Product publication updated"
        );
        Ok(product)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use deadpool_postgres::{Config, Runtime};
    use tokio_postgres::NoTls;

    async fn get_test_pool() -> Pool {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres@localhost:5432/ahlan_commerce".to_string());
        let mut cfg = Config::new();
        cfg.url = Some(database_url);
        cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap()
    }

    #[tokio::test]
    async fn test_create_and_list_products() {
        let pool = get_test_pool().await;

        // Clean up before test
        {
            let client = pool.get().await.unwrap();
            client
                .execute(
                    "DELETE FROM products WHERE handle IN ('super-cool-t-shirt', 'cozy-hoodie')",
                    &[],
                )
                .await
                .unwrap();
        }

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

        assert_eq!(prod2.title, "Cozy Hoodie");
        assert_eq!(prod2.handle, "cozy-hoodie");
        assert_eq!(prod2.price_cents, 4999);
        assert_eq!(prod2.inventory_quantity, 20);
        assert_eq!(prod2.published, false);
        assert_eq!(prod2.description, None);
        assert!(prod2.published_at.is_none());

        let products = catalog.list_products().await.unwrap();
        assert!(products.contains(&prod1));
        assert!(products.contains(&prod2));
    }

    #[tokio::test]
    async fn test_deterministic_creation() {
        let pool = get_test_pool().await;

        {
            let client = pool.get().await.unwrap();
            client
                .execute("DELETE FROM products WHERE handle = 'fixed-product'", &[])
                .await
                .unwrap();
        }

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
