use crate::AppState;
use crate::dto::{
    HealthResponse, ProductCreateRequest, ProductDto, ProductResponse, ProductsResponse,
    UpdatePublicationRequest, ImportJobCreateRequest, ImportJobResponse, JobDto,
};
use crate::error::AppError;
use axum::{Json, extract::{Path, State}, http::StatusCode, response::{IntoResponse, Html}};
use rootcause::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service health status", body = HealthResponse)
    )
)]
pub async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

#[utoipa::path(
    get,
    path = "/api/products",
    responses(
        (status = 200, description = "List all products", body = ProductsResponse)
    )
)]
pub async fn list_products_handler(
    State(state): State<AppState>,
) -> Result<Json<ProductsResponse>, AppError> {
    let domain_products = state.catalog.list_products().await?;
    let product_dtos = domain_products.into_iter().map(ProductDto::from).collect();
    Ok(Json(ProductsResponse {
        products: product_dtos,
    }))
}

#[utoipa::path(
    post,
    path = "/api/products",
    request_body = ProductCreateRequest,
    responses(
        (status = 201, description = "Created product", body = ProductResponse)
    )
)]
pub async fn create_product_handler(
    State(state): State<AppState>,
    Json(payload): Json<ProductCreateRequest>,
) -> Result<impl IntoResponse, AppError> {
    let domain_params = catalog::CreateProductParams::from(payload);
    let domain_product = state.catalog.create_product(domain_params).await?;
    let dto = ProductDto::from(domain_product);

    // Cache Invalidation
    let cache_key = cache::keys::storefront_product_page(&dto.handle);
    state.cache.delete(&cache_key).await;

    Ok((
        StatusCode::CREATED,
        Json(ProductResponse {
            product: dto,
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/products/published",
    responses(
        (status = 200, description = "List published products", body = ProductsResponse)
    )
)]
pub async fn list_published_products_handler(
    State(state): State<AppState>,
) -> Result<Json<ProductsResponse>, AppError> {
    let domain_products = state.catalog.list_published_products().await?;
    let product_dtos = domain_products.into_iter().map(ProductDto::from).collect();
    Ok(Json(ProductsResponse {
        products: product_dtos,
    }))
}

#[utoipa::path(
    patch,
    path = "/api/products/{id}/publication",
    request_body = UpdatePublicationRequest,
    params(
        ("id" = String, Path, description = "Product ID")
    ),
    responses(
        (status = 200, description = "Updated product publication status", body = ProductResponse)
    )
)]
pub async fn update_product_publication_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdatePublicationRequest>,
) -> Result<Json<ProductResponse>, AppError> {
    let domain_product = state
        .catalog
        .update_product_publication(&id, payload.published)
        .await?;
    let dto = ProductDto::from(domain_product);

    // Cache Invalidation
    let cache_key = cache::keys::storefront_product_page(&dto.handle);
    state.cache.delete(&cache_key).await;

    Ok(Json(ProductResponse {
        product: dto,
    }))
}

pub async fn create_import_job_handler(
    State(state): State<AppState>,
    Json(payload): Json<ImportJobCreateRequest>,
) -> Result<impl IntoResponse, AppError> {
    let domain_job = state.catalog.enqueue_import_job(payload.input_path).await?;
    let dto = JobDto {
        id: domain_job.id,
        status: domain_job.status,
    };
    Ok((
        StatusCode::ACCEPTED,
        Json(ImportJobResponse {
            job: dto,
        }),
    ))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorefrontPageCache {
    pub html: String,
    pub product_id: String,
    pub product_updated_at: DateTime<Utc>,
    pub rendered_at: DateTime<Utc>,
}

pub async fn storefront_product_handler(
    State(state): State<AppState>,
    Path(handle): Path<String>,
) -> Result<Html<String>, AppError> {
    let cache_key = cache::keys::storefront_product_page(&handle);

    if let Some(cached) = state.cache.get::<StorefrontPageCache>(&cache_key).await {
        return Ok(Html(cached.html));
    }

    let product_opt = state.catalog.get_product_by_handle(&handle).await?;
    let product = match product_opt {
        Some(p) if p.published => p,
        _ => return Err(AppError::NotFound("Product not found".to_string())),
    };

    let inventory_msg = if product.inventory_quantity > 0 {
        format!("{} in stock", product.inventory_quantity)
    } else {
        "Out of stock".to_string()
    };

    let price_str = format!("${:.2}", product.price_cents as f64 / 100.0);

    let html = format!(
        "<!doctype html>\n<html><head><title>{}</title></head><body><h1>{}</h1><p>{}</p><p>{}</p></body></html>",
        product.title, product.title, price_str, inventory_msg
    );

    let payload = StorefrontPageCache {
        html: html.clone(),
        product_id: product.id.0.clone(),
        product_updated_at: product.updated_at,
        rendered_at: Utc::now(),
    };

    state.cache.set(&cache_key, &payload, 300).await;

    Ok(Html(html))
}

pub async fn simulate_error_handler() -> Result<StatusCode, AppError> {
    let source_err = std::io::Error::new(
        std::io::ErrorKind::ConnectionRefused,
        "Connection refused (os error 10061)",
    );
    let report = Report::new_sendsync(source_err)
        .context("Failed to connect to Postgres database at localhost:5432")
        .attach("service = catalog-db")
        .attach("pool_size = 10")
        .into_dynamic();

    Err(AppError::Internal(report))
}

pub async fn fallback_handler() -> impl IntoResponse {
    AppError::NotFound("The requested resource does not exist.".to_string())
}
