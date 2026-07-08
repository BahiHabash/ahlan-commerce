use catalog::CreateProductParams;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq)]
pub enum AdapterError {
    InvalidPriceFormat,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExternalProduct {
    pub external_id: String,
    pub name: String,
    pub slug: String,
    pub price: String,
    #[serde(default)]
    pub stock: u32,
    #[serde(default)]
    pub is_visible: bool,
}

pub fn map_external_product(external_product: &ExternalProduct) -> Result<CreateProductParams, AdapterError> {
    let price_val: f64 = external_product.price.parse().map_err(|_| AdapterError::InvalidPriceFormat)?;
    let price_cents = (price_val * 100.0).round() as u32;

    Ok(CreateProductParams {
        title: external_product.name.clone(),
        handle: external_product.slug.clone(),
        price_cents,
        inventory_quantity: external_product.stock,
        published: external_product.is_visible,
        description: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_payload_maps_correctly() {
        let json_payload = r#"{
            "external_id": "ext_1001",
            "name": "Coffee Mug",
            "slug": "coffee-mug",
            "price": "25.00",
            "stock": 12,
            "is_visible": true
        }"#;
        
        let external_product: ExternalProduct = serde_json::from_str(json_payload).expect("Failed to parse fixture");

        let mapped = map_external_product(&external_product).expect("Failed to map");

        assert_eq!(mapped.title, "Coffee Mug");
        assert_eq!(mapped.handle, "coffee-mug");
        assert_eq!(mapped.price_cents, 2500);
        assert_eq!(mapped.inventory_quantity, 12);
        assert_eq!(mapped.published, true);
    }

    #[test]
    fn test_missing_optional_fields_use_defaults() {
        let json_payload = r#"{
            "external_id": "ext_1002",
            "name": "Tea Cup",
            "slug": "tea-cup",
            "price": "15.00"
        }"#;
        
        let external_product: ExternalProduct = serde_json::from_str(json_payload).expect("Failed to parse fixture");

        let mapped = map_external_product(&external_product).expect("Failed to map");

        assert_eq!(mapped.inventory_quantity, 0);
        assert_eq!(mapped.published, false);
    }

    #[test]
    fn test_unparseable_price_returns_error() {
        let external_product = ExternalProduct {
            external_id: "ext_1002".to_string(),
            name: "Faulty Mug".to_string(),
            slug: "faulty-mug".to_string(),
            price: "invalid_price".to_string(),
            stock: 0,
            is_visible: false,
        };

        let result = map_external_product(&external_product);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AdapterError::InvalidPriceFormat);
    }
}
