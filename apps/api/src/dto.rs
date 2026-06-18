use catalog::Product as DomainProduct;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ProductCreateRequest {
    pub title: String,
    pub handle: String,
    pub price_cents: u32,
    pub inventory_quantity: u32,
    pub published: bool,
    pub description: Option<String>,
}

impl From<ProductCreateRequest> for catalog::CreateProductParams {
    fn from(req: ProductCreateRequest) -> Self {
        Self {
            title: req.title,
            handle: req.handle,
            price_cents: req.price_cents,
            inventory_quantity: req.inventory_quantity,
            published: req.published,
            description: req.description,
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct ProductDto {
    pub id: String,
    pub title: String,
    pub handle: String,
    pub price_cents: u32,
    pub inventory_quantity: u32,
    pub published: bool,
    pub description: Option<String>,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<DomainProduct> for ProductDto {
    fn from(p: DomainProduct) -> Self {
        Self {
            id: p.id.0,
            title: p.title,
            handle: p.handle,
            price_cents: p.price_cents,
            inventory_quantity: p.inventory_quantity,
            published: p.published,
            description: p.description,
            published_at: p.published_at,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct ProductResponse {
    pub product: ProductDto,
}

#[derive(Debug, serde::Serialize)]
pub struct ProductsResponse {
    pub products: Vec<ProductDto>,
}

#[derive(Debug, serde::Serialize)]
pub struct HealthResponse {
    pub status: String,
}
