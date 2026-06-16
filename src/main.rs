mod catalog;

use axum::{
    extract::State,
    http::StatusCode,
    routing::get,
    Json, Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use catalog::{Catalog, Product, ProductCreate};

#[derive(Clone)]
struct AppState {
    catalog: Arc<RwLock<Catalog>>,
}

#[derive(serde::Serialize)]
struct HealthResponse {
    status: String,
}

#[derive(serde::Serialize)]
struct ProductsResponse {
    products: Vec<Product>,
}

#[derive(serde::Serialize)]
struct ProductResponse {
    product: Product,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        catalog: Arc::new(RwLock::new(Catalog::new())),
    };

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/api/products", get(list_products_handler).post(create_product_handler))
        .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Ahlan Commerce Catalog System starting up on port 3000...");
    axum::serve(listener, app).await.unwrap();
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

async fn list_products_handler(State(state): State<AppState>) -> Json<ProductsResponse> {
    let catalog = state.catalog.read().await;
    Json(ProductsResponse {
        products: catalog.list_products(),
    })
}

async fn create_product_handler(
    State(state): State<AppState>,
    Json(input): Json<ProductCreate>,
) -> (StatusCode, Json<ProductResponse>) {
    let mut catalog = state.catalog.write().await;
    let product = catalog.create_product(input);
    (StatusCode::CREATED, Json(ProductResponse { product }))
}
