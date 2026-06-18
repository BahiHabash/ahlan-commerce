pub mod config;
pub mod dto;
pub mod error;
pub mod handlers;
pub mod routes;

use std::sync::Arc;
use tokio::sync::RwLock;
use axum::{
    routing::get,
    Router,
    extract::Request,
    middleware::{self, Next},
    response::Response,
};
use catalog::{Catalog, RealClock, RealIdGenerator};
use config::Config;
use routes::{HEALTH_ROUTE, PRODUCTS_ROUTE};
use handlers::{health_handler, list_products_handler, create_product_handler};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub catalog: Arc<RwLock<Catalog>>,
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

    // Initialize catalog with real clock and real ID generator
    let clock = Arc::new(RealClock);
    let id_generator = Arc::new(RealIdGenerator);
    let catalog = Catalog::new(clock, id_generator);

    let state = AppState {
        config: config.clone(),
        catalog: Arc::new(RwLock::new(catalog)),
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

    fn test_app() -> Router {
        let fixed_time = chrono::Utc.with_ymd_and_hms(2026, 6, 17, 12, 0, 0).unwrap();
        let clock = Arc::new(TestClock::new(fixed_time));
        let id_generator = Arc::new(TestIdGenerator::new(vec![
            "test-id-123".to_string(),
        ]));
        let catalog = Catalog::new(clock, id_generator);

        let state = AppState {
            config: Config {
                port: 3000,
                env: "test".to_string(),
            },
            catalog: Arc::new(RwLock::new(catalog)),
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
            .route(routes::SIMULATE_ERROR_ROUTE, get(handlers::simulate_error_handler))
            .fallback(handlers::fallback_handler)
            .layer(trace_layer)
            .layer(middleware::from_fn(request_id_middleware))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = test_app();

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
    async fn test_create_and_list_products() {
        let app = test_app();

        // 1. Create a product
        let create_payload = json!({
            "title": "Test Hoodie",
            "handle": "test-hoodie",
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
        assert_eq!(body["product"]["handle"], "test-hoodie");
        assert_eq!(body["product"]["price_cents"], 3500);
        assert_eq!(body["product"]["inventory_quantity"], 10);
        assert_eq!(body["product"]["published"], true);
        assert_eq!(body["product"]["created_at"], "2026-06-17T12:00:00Z");
        assert_eq!(body["product"]["updated_at"], "2026-06-17T12:00:00Z");

        // 2. List products
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
        assert_eq!(products.len(), 1);
        assert_eq!(products[0]["id"], "test-id-123");
        assert_eq!(products[0]["title"], "Test Hoodie");
        assert_eq!(products[0]["created_at"], "2026-06-17T12:00:00Z");
        assert_eq!(products[0]["updated_at"], "2026-06-17T12:00:00Z");
    }

    #[tokio::test]
    async fn test_validation_errors() {
        let app = test_app();

        // 1. Empty title validation
        let invalid_title_payload = json!({
            "title": "",
            "handle": "test-hoodie",
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
    async fn test_duplicate_handle_error() {
        let app = test_app();

        let payload = json!({
            "title": "Test Hoodie",
            "handle": "test-hoodie",
            "price_cents": 3500,
            "inventory_quantity": 10,
            "published": true
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
        assert_eq!(body["error"]["message"], "Another product already uses this handle: test-hoodie");
        assert!(body["error"]["request_id"].is_string());
    }

    #[tokio::test]
    async fn test_not_found_fallback() {
        let app = test_app();

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
        let app = test_app();

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
}

