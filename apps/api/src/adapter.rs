use catalog::CreateProductParams;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ExternalProduct {
    pub external_id: String,
    pub name: String,
    pub slug: String,
    pub price: String,
    pub stock: u32,
    pub is_visible: bool,
}

pub fn map_external_product(ext: &ExternalProduct) -> Result<CreateProductParams, String> {
    let price_val: f64 = ext.price.parse().map_err(|_| "Invalid price format")?;
    let price_cents = (price_val * 100.0).round() as u32;

    Ok(CreateProductParams {
        title: ext.name.clone(),
        handle: ext.slug.clone(),
        price_cents,
        inventory_quantity: ext.stock,
        published: ext.is_visible,
        description: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_map_external_product() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.pop();
        d.pop();
        d.push("fixtures/external-product.json");

        let file_contents = fs::read_to_string(d).expect("Failed to read fixture");
        let ext: ExternalProduct = serde_json::from_str(&file_contents).expect("Failed to parse fixture");

        let mapped = map_external_product(&ext).expect("Failed to map");

        assert_eq!(mapped.title, "Coffee Mug");
        assert_eq!(mapped.handle, "coffee-mug");
        assert_eq!(mapped.price_cents, 2500);
        assert_eq!(mapped.inventory_quantity, 12);
        assert_eq!(mapped.published, true);
    }

    #[test]
    fn test_map_external_product_invalid_price() {
        let ext = ExternalProduct {
            external_id: "ext_1002".to_string(),
            name: "Faulty Mug".to_string(),
            slug: "faulty-mug".to_string(),
            price: "invalid_price".to_string(),
            stock: 0,
            is_visible: false,
        };

        let result = map_external_product(&ext);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid price format");
    }
}
