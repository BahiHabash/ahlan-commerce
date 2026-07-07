pub mod config;
pub mod dto;
pub mod error;
pub mod graphql;
pub mod handlers;
pub mod routes;
pub mod openapi;

#[derive(Clone)]
pub struct AppState {
    pub config: config::Config,
    pub catalog: catalog::Catalog,
    pub schema: graphql::AppSchema,
    pub cache: cache::Cache,
}

