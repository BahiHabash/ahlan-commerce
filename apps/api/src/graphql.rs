use async_graphql::{Context, EmptySubscription, Error, ErrorExtensions, InputObject, InputValueError, InputValueResult, Object, Scalar, ScalarType, SimpleObject, Value, ID};
use catalog::{Catalog, CatalogError, CreateProductParams};
use chrono::{DateTime as ChronoDateTime, Utc};
use serde::{Deserialize, Serialize};

/// ISO-8601 UTC timestamp string, for example "2026-06-13T10:30:00Z".
#[derive(Serialize, Deserialize)]
pub struct DateTime(ChronoDateTime<Utc>);

#[Scalar(name = "DateTime")]
impl ScalarType for DateTime {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(s) = &value {
            Ok(DateTime(
                s.parse::<ChronoDateTime<Utc>>()
                    .map_err(|_| InputValueError::custom("Invalid ISO-8601 UTC timestamp"))?
            ))
        } else {
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
    }
}

#[derive(SimpleObject)]
pub struct Product {
    pub id: ID,
    pub title: String,
    pub handle: String,
    pub description: Option<String>,
    #[graphql(name = "priceCents")]
    pub price_cents: i32,
    #[graphql(name = "inventoryQuantity")]
    pub inventory_quantity: i32,
    pub published: bool,
    #[graphql(name = "publishedAt")]
    pub published_at: Option<DateTime>,
    #[graphql(name = "createdAt")]
    pub created_at: DateTime,
    #[graphql(name = "updatedAt")]
    pub updated_at: DateTime,
}

impl From<catalog::Product> for Product {
    fn from(p: catalog::Product) -> Self {
        Self {
            id: ID(p.id.0),
            title: p.title,
            handle: p.handle,
            description: p.description,
            price_cents: p.price_cents as i32,
            inventory_quantity: p.inventory_quantity as i32,
            published: p.published,
            published_at: p.published_at.map(DateTime),
            created_at: DateTime(p.created_at),
            updated_at: DateTime(p.updated_at),
        }
    }
}

#[derive(InputObject)]
pub struct ProductCreateInput {
    pub title: String,
    pub handle: String,
    pub description: Option<String>,
    #[graphql(name = "priceCents")]
    pub price_cents: i32,
    #[graphql(name = "inventoryQuantity")]
    pub inventory_quantity: i32,
    pub published: bool,
}

impl From<ProductCreateInput> for CreateProductParams {
    fn from(input: ProductCreateInput) -> Self {
        Self {
            title: input.title,
            handle: input.handle,
            description: input.description,
            price_cents: input.price_cents as u32,
            inventory_quantity: input.inventory_quantity as u32,
            published: input.published,
        }
    }
}

fn map_catalog_error(err: CatalogError) -> Error {
    let request_id = crate::error::REQUEST_ID.try_with(|id| id.clone())
        .unwrap_or_else(|_| uuid::Uuid::now_v7().to_string());

    let (code, message) = match &err {
        CatalogError::EmptyTitle => ("validation_failed", "Product title is required.".to_string()),
        CatalogError::EmptyHandle => ("validation_failed", "Product handle is required.".to_string()),
        CatalogError::EmptyInputPath => ("validation_failed", "Input path is required.".to_string()),
        CatalogError::DuplicateHandle { handle } => (
            "duplicate_product_handle",
            format!("Another product already uses this handle: {}", handle),
        ),
        CatalogError::ProductNotFound { id } => ("not_found", format!("Product not found: {}", id)),
        CatalogError::Database(_) | CatalogError::Pool(_) => {
            ("internal_error", "The server encountered an unexpected error.".to_string())
        }
    };

    if code == "internal_error" {
        tracing::error!(
            request_id = %request_id,
            code = %code,
            error = ?err,
            "Server error occurred in GraphQL resolver"
        );
    } else {
        tracing::warn!(
            request_id = %request_id,
            code = %code,
            message = %message,
            "Client error occurred in GraphQL resolver"
        );
    }

    Error::new(message).extend_with(|_, e| {
        e.set("code", code);
        e.set("request_id", request_id);
    })
}

pub struct Query;

#[Object]
impl Query {
    async fn products(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Product>> {
        let catalog = ctx.data::<Catalog>().map_err(|_| Error::new("Internal Server Error"))?;
        let products = catalog.list_products().await.map_err(map_catalog_error)?;
        Ok(products.into_iter().map(Into::into).collect())
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    async fn product_create(
        &self,
        ctx: &Context<'_>,
        input: ProductCreateInput,
    ) -> async_graphql::Result<Product> {
        let catalog = ctx.data::<Catalog>().map_err(|_| Error::new("Internal Server Error"))?;
        let product = catalog.create_product(input.into()).await.map_err(map_catalog_error)?;
        Ok(product.into())
    }
}

pub type AppSchema = async_graphql::Schema<Query, Mutation, EmptySubscription>;

pub fn build_schema(catalog: Catalog) -> AppSchema {
    async_graphql::Schema::build(Query, Mutation, EmptySubscription)
        .data(catalog)
        .finish()
}

use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::State;

pub async fn graphql_handler(
    State(state): State<crate::AppState>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    state.schema.execute(req.into_inner()).await.into()
}
