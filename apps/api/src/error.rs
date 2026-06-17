use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use uuid::Uuid;

tokio::task_local! {
    pub static REQUEST_ID: String;
}

#[derive(Debug)]
pub enum AppError {
    Validation(String),
    DuplicateProductHandle(String),
    NotFound(String),
    DependencyUnavailable(rootcause::Report),
    Internal(rootcause::Report),
}

#[derive(Serialize)]
struct ErrorEnvelope {
    error: ErrorDetails,
}

#[derive(Serialize)]
struct ErrorDetails {
    code: &'static str,
    message: String,
    request_id: String,
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::DuplicateProductHandle(_) => StatusCode::CONFLICT,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::DependencyUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::Validation(_) => "validation_failed",
            AppError::DuplicateProductHandle(_) => "duplicate_product_handle",
            AppError::NotFound(_) => "not_found",
            AppError::DependencyUnavailable(_) => "dependency_unavailable",
            AppError::Internal(_) => "internal_error",
        }
    }

    pub fn message(&self) -> String {
        match self {
            AppError::Validation(msg) => msg.clone(),
            AppError::DuplicateProductHandle(msg) => msg.clone(),
            AppError::NotFound(msg) => msg.clone(),
            AppError::DependencyUnavailable(_) => {
                "A required dependency is unavailable. Please try again later.".to_string()
            }
            AppError::Internal(_) => "The server encountered an unexpected error.".to_string(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.error_code();
        let message = self.message();
        
        let request_id = REQUEST_ID.try_with(|id| id.clone())
            .unwrap_or_else(|_| Uuid::now_v7().to_string());

        // For internal/dependency unavailable errors, log the root cause!
        if status.is_server_error() {
            match &self {
                AppError::Internal(report) | AppError::DependencyUnavailable(report) => {
                    eprintln!(
                        "[ERROR] request_id={} code={} error={:?}",
                        request_id, code, report
                    );
                }
                _ => {}
            }
        } else {
            println!(
                "[WARN] request_id={} code={} message={}",
                request_id, code, message
            );
        }

        let envelope = ErrorEnvelope {
            error: ErrorDetails {
                code,
                message,
                request_id,
            },
        };

        (status, Json(envelope)).into_response()
    }
}

impl From<catalog::CatalogError> for AppError {
    fn from(err: catalog::CatalogError) -> Self {
        match err {
            catalog::CatalogError::EmptyTitle => {
                AppError::Validation("Product title is required.".to_string())
            }
            catalog::CatalogError::EmptyHandle => {
                AppError::Validation("Product handle is required.".to_string())
            }
            catalog::CatalogError::DuplicateHandle { handle } => {
                AppError::DuplicateProductHandle(format!(
                    "Another product already uses this handle: {}",
                    handle
                ))
            }
        }
    }
}
