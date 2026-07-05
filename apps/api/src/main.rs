pub mod config;
pub mod dto;
pub mod error;
pub mod graphql;
pub mod handlers;
pub mod routes;

use std::sync::Arc;
use axum::{
    routing::get,
    Router,
    extract::Request,
    middleware::{self, Next},
    response::Response,
};
use crate::graphql::AppSchema;
use catalog::{Catalog, RealClock, RealIdGenerator};
use config::Config;
use routes::{HEALTH_ROUTE, PRODUCTS_ROUTE, PUBLISHED_PRODUCTS_ROUTE, PRODUCT_PUBLICATION_ROUTE, IMPORT_JOBS_ROUTE};
use handlers::{health_handler, list_products_handler, create_product_handler,
    list_published_products_handler, update_product_publication_handler, create_import_job_handler};
use deadpool_postgres::{Config as DbConfig, Runtime};
use tokio_postgres::NoTls;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub catalog: Catalog,
    pub schema: AppSchema,
}

async fn request_id_middleware(mut req: Request, next: Next) -> Response {
    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::now_v7().to_string());

    // Insert into request headers so down-stream layers/handlers can access it
    req.headers_mut().insert(
        "x-request-id",
        axum::http::HeaderValue::from_str(&request_id).unwrap(),
    );

    error::REQUEST_ID.scope(request_id.clone(), async move {
        let mut response = next.run(req).await;
        response.headers_mut().insert(
            "x-request-id",
            axum::http::HeaderValue::from_str(&request_id).unwrap(),
        );
        response
    }).await
}

#[tokio::main]
async fn main() {
    // Load config once at startup
    let config = Config::from_env();

    // Initialize tracing subscriber with span close events enabled
    tracing_subscriber::fmt()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api=info,catalog=info,tower_http=info".into()),
        )
        .init();

    // Initialize database pool (deadpool-postgres)
    let mut db_cfg = DbConfig::new();
    db_cfg.url = Some(config.database_url.clone());
    let pool = db_cfg
        .create_pool(Some(Runtime::Tokio1), NoTls)
        .expect("Failed to create PostgreSQL connection pool");

    // Eagerly verify the connection is usable.
    let _ = pool.get()
        .await
        .expect("Failed to connect to PostgreSQL database");

    // Initialize catalog with pool, real clock, and real ID generator
    let clock = Arc::new(RealClock);
    let id_generator = Arc::new(RealIdGenerator);
    let catalog = Catalog::new(pool, clock, id_generator);

    let schema = crate::graphql::build_schema(catalog.clone());

    let state = AppState {
        config: config.clone(),
        catalog,
        schema,
    };

    // Set up tower-http TraceLayer for request tracing
    let trace_layer = tower_http::trace::TraceLayer::new_for_http()
        .make_span_with(|request: &Request<_>| {
            let request_id = request.headers()
                .get("x-request-id")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            
            let matched_path = request
                .extensions()
                .get::<axum::extract::MatchedPath>()
                .map(|mp| mp.as_str())
                .unwrap_or_else(|| request.uri().path());

            tracing::info_span!(
                "request",
                request_id = %request_id,
                method = %request.method(),
                route = %matched_path,
                status = tracing::field::Empty,
                latency_ms = tracing::field::Empty,
                error_code = tracing::field::Empty,
            )
        })
        .on_response(|response: &Response, latency: std::time::Duration, span: &tracing::Span| {
            let latency_ms = latency.as_millis() as u64;
            let status = response.status().as_u16();
            
            span.record("status", status);
            span.record("latency_ms", latency_ms);
            
            if let Some(err_code) = response.extensions().get::<crate::error::ErrorCode>() {
                span.record("error_code", err_code.0);
                tracing::warn!(
                    status = status,
                    latency_ms = latency_ms,
                    error_code = err_code.0,
                    "request failed"
                );
            } else {
                tracing::info!(
                    status = status,
                    latency_ms = latency_ms,
                    "request completed"
                );
            }
        });

    let app = Router::new()
        .route(HEALTH_ROUTE, get(health_handler))
        .route(
            PRODUCTS_ROUTE,
            get(list_products_handler).post(create_product_handler),
        )
        .route(PUBLISHED_PRODUCTS_ROUTE, get(list_published_products_handler))
        .route(PRODUCT_PUBLICATION_ROUTE, axum::routing::patch(update_product_publication_handler))
        .route(IMPORT_JOBS_ROUTE, axum::routing::post(create_import_job_handler))
        .route(routes::GRAPHQL_ROUTE, axum::routing::post(crate::graphql::graphql_handler))
        .route(routes::SIMULATE_ERROR_ROUTE, get(handlers::simulate_error_handler))
        .fallback(handlers::fallback_handler)
        .layer(trace_layer)
        .layer(middleware::from_fn(request_id_middleware))
        .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    
    tracing::info!(
        port = config.port,
        env = %config.env,
        addr = %addr,
        "Ahlan Commerce Catalog API starting up"
    );
    
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt; // for `oneshot`
    use serde_json::{json, Value};
    use catalog::{TestClock, TestIdGenerator};
    use chrono::TimeZone;

    async fn test_app() -> Router {
        test_app_with_id("test-id-123".to_string()).await
    }

    async fn test_app_with_id(initial_id: String) -> Router {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres@localhost:5432/ahlan_commerce".to_string());

        let mut db_cfg = DbConfig::new();
        db_cfg.url = Some(database_url.clone());
        let pool = db_cfg
            .create_pool(Some(Runtime::Tokio1), NoTls)
            .expect("Failed to create test pool");

        // Truncate products and import_jobs table for integration testing
        {
            let client = pool.get().await.unwrap();
            client.execute("TRUNCATE TABLE products, import_jobs", &[]).await.unwrap();
        }

        let fixed_time = chrono::Utc.with_ymd_and_hms(2026, 6, 17, 12, 0, 0).unwrap();
        let clock = Arc::new(TestClock::new(fixed_time));
        let id_generator = Arc::new(TestIdGenerator::new(vec![
            initial_id,
        ]));
        let catalog = Catalog::new(pool, clock, id_generator);

        let schema = crate::graphql::build_schema(catalog.clone());

        let state = AppState {
            config: Config {
                port: 3000,
                env: "test".to_string(),
                database_url,
            },
            catalog,
            schema,
        };

        let trace_layer = tower_http::trace::TraceLayer::new_for_http()
            .make_span_with(|request: &Request<_>| {
                let request_id = request.headers()
                    .get("x-request-id")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("");
                
                let matched_path = request
                    .extensions()
                    .get::<axum::extract::MatchedPath>()
                    .map(|mp| mp.as_str())
                    .unwrap_or_else(|| request.uri().path());

                tracing::info_span!(
                    "request",
                    request_id = %request_id,
                    method = %request.method(),
                    route = %matched_path,
                    status = tracing::field::Empty,
                    latency_ms = tracing::field::Empty,
                    error_code = tracing::field::Empty,
                )
            })
            .on_response(|response: &Response, latency: std::time::Duration, span: &tracing::Span| {
                let latency_ms = latency.as_millis() as u64;
                let status = response.status().as_u16();
                
                span.record("status", status);
                span.record("latency_ms", latency_ms);
                
                if let Some(err_code) = response.extensions().get::<crate::error::ErrorCode>() {
                    span.record("error_code", err_code.0);
                    tracing::warn!(
                        status = status,
                        latency_ms = latency_ms,
                        error_code = err_code.0,
                        "request failed"
                    );
                } else {
                    tracing::info!(
                        status = status,
                        latency_ms = latency_ms,
                        "request completed"
                    );
                }
            });

        Router::new()
            .route(HEALTH_ROUTE, get(health_handler))
            .route(
                PRODUCTS_ROUTE,
                get(list_products_handler).post(create_product_handler),
            )
            .route(PUBLISHED_PRODUCTS_ROUTE, get(list_published_products_handler))
            .route(PRODUCT_PUBLICATION_ROUTE, axum::routing::patch(update_product_publication_handler))
            .route(IMPORT_JOBS_ROUTE, axum::routing::post(create_import_job_handler))
            .route(routes::GRAPHQL_ROUTE, axum::routing::post(crate::graphql::graphql_handler))
            .route(routes::SIMULATE_ERROR_ROUTE, get(handlers::simulate_error_handler))
            .fallback(handlers::fallback_handler)
            .layer(trace_layer)
            .layer(middleware::from_fn(request_id_middleware))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri(HEALTH_ROUTE)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 10000).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, json!({ "status": "ok" }));
    }

    #[tokio::test]
    async fn test_prd_prod_001_valid_create() {
        let app = test_app().await;

        let create_payload = json!({
            "title": "Test Hoodie",
            "handle": "test-hoodie-create",
            "price_cents": 3500,
            "inventory_quantity": 10,
            "published": true,
            "description": "This is a test hoodie"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(PRODUCTS_ROUTE)
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&create_payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(response.into_body(), 10000).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(body["product"]["id"], "test-id-123");
        assert_eq!(body["product"]["title"], "Test Hoodie");
        assert_eq!(body["product"]["handle"], "test-hoodie-create");
        assert_eq!(body["product"]["price_cents"], 3500);
        assert_eq!(body["product"]["inventory_quantity"], 10);
        assert_eq!(body["product"]["published"], true);
        assert_eq!(body["product"]["description"], "This is a test hoodie");
        assert_eq!(body["product"]["published_at"], "2026-06-17T12:00:00Z");
        assert_eq!(body["product"]["created_at"], "2026-06-17T12:00:00Z");
        assert_eq!(body["product"]["updated_at"], "2026-06-17T12:00:00Z");
    }

    #[tokio::test]
    async fn test_prd_prod_003_list_empty_products() {
        let app = test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri(PRODUCTS_ROUTE)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 10000).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        
        let products = body["products"].as_array().unwrap();
        assert!(products.is_empty());
    }

    #[tokio::test]
    async fn test_prd_prod_004_list_persisted_products() {
        let app = test_app().await;

        let create_payload = json!({
            "title": "Test Hoodie",
            "handle": "test-hoodie-list",
            "price_cents": 3500,
            "inventory_quantity": 10,
            "published": true,
            "description": "This is a test hoodie"
        });

        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(PRODUCTS_ROUTE)
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&create_payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri(PRODUCTS_ROUTE)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 10000).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();

        let products = body["products"].as_array().unwrap();
        let test_product = products.iter().find(|p| p["id"] == "test-id-123")
            .expect("Product with test-id-123 should be present in list");
        assert_eq!(test_product["title"], "Test Hoodie");
        assert_eq!(test_product["handle"], "test-hoodie-list");
        assert_eq!(test_product["description"], "This is a test hoodie");
        assert_eq!(test_product["published_at"], "2026-06-17T12:00:00Z");
        assert_eq!(test_product["created_at"], "2026-06-17T12:00:00Z");
        assert_eq!(test_product["updated_at"], "2026-06-17T12:00:00Z");
    }

    #[tokio::test]
    async fn test_prd_prod_005_invalid_create_input_rejected() {
        let app = test_app().await;

        // 1. Empty title validation
        let invalid_title_payload = json!({
            "title": "",
            "handle": "test-hoodie-validation",
            "price_cents": 3500,
            "inventory_quantity": 10,
            "published": true
        });
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(PRODUCTS_ROUTE)
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&invalid_title_payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = axum::body::to_bytes(response.into_body(), 10000).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["error"]["code"], "validation_failed");
        assert_eq!(body["error"]["message"], "Product title is required.");
        assert!(body["error"]["request_id"].is_string());

        // 2. Empty handle validation
        let invalid_handle_payload = json!({
            "title": "Hoodie",
            "handle": "  ",
            "price_cents": 3500,
            "inventory_quantity": 10,
            "published": true
        });
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(PRODUCTS_ROUTE)
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&invalid_handle_payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = axum::body::to_bytes(response.into_body(), 10000).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["error"]["code"], "validation_failed");
        assert_eq!(body["error"]["message"], "Product handle is required.");
        assert!(body["error"]["request_id"].is_string());
    }

    #[tokio::test]
    async fn test_prd_prod_002_duplicate_handle_rejected() {
        let app = test_app().await;

        let payload = json!({
            "title": "Test Hoodie",
            "handle": "test-hoodie-dup",
            "price_cents": 3500,
            "inventory_quantity": 10,
            "published": false,
            "description": null
        });

        // First creation succeeds
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(PRODUCTS_ROUTE)
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(response.into_body(), 10000).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["product"]["description"], serde_json::Value::Null);
        assert_eq!(body["product"]["published_at"], serde_json::Value::Null);

        // Second creation with same handle fails
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(PRODUCTS_ROUTE)
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = axum::body::to_bytes(response.into_body(), 10000).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["error"]["code"], "duplicate_product_handle");
        assert_eq!(body["error"]["message"], "Another product already uses this handle: test-hoodie-dup");
        assert!(body["error"]["request_id"].is_string());
    }

    #[tokio::test]
    async fn test_not_found_fallback() {
        let app = test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/nonexistent-route-path")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = axum::body::to_bytes(response.into_body(), 10000).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body["error"]["code"], "not_found");
        assert_eq!(body["error"]["message"], "The requested resource does not exist.");
        assert!(body["error"]["request_id"].is_string());
    }

    #[tokio::test]
    async fn test_simulated_internal_error() {
        let app = test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/simulate-error")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = axum::body::to_bytes(response.into_body(), 10000).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        
        // Response envelope checks
        assert_eq!(body["error"]["code"], "internal_error");
        assert_eq!(body["error"]["message"], "The server encountered an unexpected error.");
        assert!(body["error"]["request_id"].is_string());
        
        // Assert that private/sensitive details do NOT leak in the response body!
        let body_str = serde_json::to_string(&body).unwrap();
        assert!(!body_str.contains("Postgres"));
        assert!(!body_str.contains("localhost:5432"));
        assert!(!body_str.contains("ConnectionRefused"));
        assert!(!body_str.contains("catalog-db"));
    }

    #[tokio::test]
    async fn test_graphql_products_query_and_mutation() {
        let app = test_app().await;

        let mutation = r#"
            mutation {
                productCreate(input: {
                    title: "GraphQL Hoodie"
                    handle: "graphql-hoodie"
                    priceCents: 4500
                    inventoryQuantity: 20
                    published: true
                    description: "Created via GraphQL"
                }) {
                    id
                    title
                    handle
                    priceCents
                    inventoryQuantity
                    published
                    description
                    createdAt
                    updatedAt
                    publishedAt
                }
            }
        "#;

        let mutation_payload = json!({
            "query": mutation
        });

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(routes::GRAPHQL_ROUTE)
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&mutation_payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), 10000).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        
        let product = &body["data"]["productCreate"];
        assert_eq!(product["id"], "test-id-123");
        assert_eq!(product["title"], "GraphQL Hoodie");
        assert_eq!(product["handle"], "graphql-hoodie");
        assert_eq!(product["priceCents"], 4500);
        assert_eq!(product["inventoryQuantity"], 20);
        assert_eq!(product["published"], true);
        assert_eq!(product["description"], "Created via GraphQL");
        assert_eq!(product["createdAt"], "2026-06-17T12:00:00Z");
        assert_eq!(product["updatedAt"], "2026-06-17T12:00:00Z");
        assert_eq!(product["publishedAt"], "2026-06-17T12:00:00Z");

        // Query test
        let query = r#"
            query {
                products {
                    id
                    title
                    handle
                }
            }
        "#;

        let query_payload = json!({
            "query": query
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(routes::GRAPHQL_ROUTE)
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&query_payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), 10000).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        
        let products = body["data"]["products"].as_array().unwrap();
        let test_product = products.iter().find(|p| p["id"] == "test-id-123").unwrap();
        assert_eq!(test_product["title"], "GraphQL Hoodie");
        assert_eq!(test_product["handle"], "graphql-hoodie");
    }

    #[tokio::test]
    async fn test_import_job_valid_create() {
        let app = test_app_with_id("018e69d0-0000-7000-0000-000000000000".to_string()).await;

        let payload = json!({
            "input_path": "fixtures/products.json"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(IMPORT_JOBS_ROUTE)
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
        let body = axum::body::to_bytes(response.into_body(), 10000).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(body["job"]["status"], "queued");
        assert!(body["job"]["id"].is_string());
    }

    #[tokio::test]
    async fn test_import_job_invalid_create() {
        let app = test_app().await;

        let payload = json!({
            "input_path": "   "
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(IMPORT_JOBS_ROUTE)
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = axum::body::to_bytes(response.into_body(), 10000).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(body["error"]["code"], "validation_failed");
        assert_eq!(body["error"]["message"], "Input path is required.");
    }
}

