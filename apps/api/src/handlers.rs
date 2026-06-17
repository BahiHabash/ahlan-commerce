use crate::AppState;
use crate::dto::{
    HealthResponse, ProductCreateRequest, ProductDto, ProductResponse, ProductsResponse,
};
use crate::error::AppError;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use rootcause::prelude::*;

pub async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

pub async fn list_products_handler(State(state): State<AppState>) -> Json<ProductsResponse> {
    let catalog = state.catalog.read().await;
    let domain_products = catalog.list_products();
    let product_dtos = domain_products.into_iter().map(ProductDto::from).collect();
    Json(ProductsResponse {
        products: product_dtos,
    })
}

pub async fn create_product_handler(
    State(state): State<AppState>,
    Json(payload): Json<ProductCreateRequest>,
) -> Result<impl IntoResponse, AppError> {
    let mut catalog = state.catalog.write().await;
    let domain_params = catalog::CreateProductParams::from(payload);
    let domain_product = catalog.create_product(domain_params)?;
    let product_dto = ProductDto::from(domain_product);
    Ok((
        StatusCode::CREATED,
        Json(ProductResponse {
            product: product_dto,
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
