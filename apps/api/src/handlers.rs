use crate::AppState;
use crate::dto::{
    HealthResponse, ProductCreateRequest, ProductDto, ProductResponse, ProductsResponse,
    UpdatePublicationRequest, ImportJobCreateRequest, ImportJobResponse, JobDto,
};
use crate::error::AppError;
use axum::{Json, extract::{Path, State}, http::StatusCode, response::IntoResponse};
use rootcause::prelude::*;

pub async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

pub async fn list_products_handler(
    State(state): State<AppState>,
) -> Result<Json<ProductsResponse>, AppError> {
    let domain_products = state.catalog.list_products().await?;
    let product_dtos = domain_products.into_iter().map(ProductDto::from).collect();
    Ok(Json(ProductsResponse {
        products: product_dtos,
    }))
}

pub async fn create_product_handler(
    State(state): State<AppState>,
    Json(payload): Json<ProductCreateRequest>,
) -> Result<impl IntoResponse, AppError> {
    let domain_params = catalog::CreateProductParams::from(payload);
    let domain_product = state.catalog.create_product(domain_params).await?;
    let product_dto = ProductDto::from(domain_product);
    Ok((
        StatusCode::CREATED,
        Json(ProductResponse {
            product: product_dto,
        }),
    ))
}

pub async fn list_published_products_handler(
    State(state): State<AppState>,
) -> Result<Json<ProductsResponse>, AppError> {
    let domain_products = state.catalog.list_published_products().await?;
    let product_dtos = domain_products.into_iter().map(ProductDto::from).collect();
    Ok(Json(ProductsResponse {
        products: product_dtos,
    }))
}

pub async fn update_product_publication_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdatePublicationRequest>,
) -> Result<Json<ProductResponse>, AppError> {
    let domain_product = state
        .catalog
        .update_product_publication(&id, payload.published)
        .await?;
    Ok(Json(ProductResponse {
        product: ProductDto::from(domain_product),
    }))
}

pub async fn create_import_job_handler(
    State(state): State<AppState>,
    Json(payload): Json<ImportJobCreateRequest>,
) -> Result<impl IntoResponse, AppError> {
    let domain_job = state.catalog.enqueue_import_job(payload.input_path).await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(ImportJobResponse {
            job: JobDto {
                id: domain_job.id,
                status: domain_job.status,
            },
        }),
    ))
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
