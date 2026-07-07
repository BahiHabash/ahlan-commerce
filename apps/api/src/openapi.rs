use utoipa::OpenApi;
use crate::dto::{
    HealthResponse, ProductCreateRequest, ProductDto, ProductResponse, ProductsResponse,
    UpdatePublicationRequest,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::health_handler,
        crate::handlers::list_products_handler,
        crate::handlers::create_product_handler,
        crate::handlers::list_published_products_handler,
        crate::handlers::update_product_publication_handler
    ),
    components(
        schemas(
            HealthResponse,
            ProductCreateRequest,
            ProductDto,
            ProductResponse,
            ProductsResponse,
            UpdatePublicationRequest
        )
    ),
    tags(
        (name = "ahlan-commerce", description = "Ahlan Commerce API")
    )
)]
pub struct ApiDoc;
