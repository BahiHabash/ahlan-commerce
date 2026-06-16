#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProductId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Product {
    pub id: ProductId,
    pub title: String,
    pub handle: String,
    pub price_cents: u32,
    pub inventory_quantity: u32,
    pub published: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductCreate {
    pub title: String,
    pub handle: String,
    pub price_cents: u32,
    pub inventory_quantity: u32,
    pub published: bool,
}

#[derive(Debug, Default)]
pub struct Catalog {
    products: Vec<Product>,
}

impl Catalog {
    pub fn new() -> Self {
        Self {
            products: Vec::new(),
        }
    }

    pub fn create_product(&mut self, input: ProductCreate) -> Product {
        let id = ProductId(uuid::Uuid::new_v4().to_string());
        let product = Product {
            id,
            title: input.title,
            handle: input.handle,
            price_cents: input.price_cents,
            inventory_quantity: input.inventory_quantity,
            published: input.published,
        };
        self.products.push(product.clone());
        product
    }

    pub fn list_products(&self) -> Vec<Product> {
        self.products.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_list_products() {
        let mut catalog = Catalog::new();

        let input1 = ProductCreate {
            title: "Super Cool T-Shirt".to_string(),
            handle: "super-cool-t-shirt".to_string(),
            price_cents: 2999,
            inventory_quantity: 50,
            published: true,
        };

        let input2 = ProductCreate {
            title: "Cozy Hoodie".to_string(),
            handle: "cozy-hoodie".to_string(),
            price_cents: 4999,
            inventory_quantity: 20,
            published: false,
        };

        let prod1 = catalog.create_product(input1);
        let prod2 = catalog.create_product(input2);

        // Verify that fields survived creation for product 1
        assert_eq!(prod1.title, "Super Cool T-Shirt");
        assert_eq!(prod1.handle, "super-cool-t-shirt");
        assert_eq!(prod1.price_cents, 2999);
        assert_eq!(prod1.inventory_quantity, 50);
        assert_eq!(prod1.published, true);

        // Verify that fields survived creation for product 2
        assert_eq!(prod2.title, "Cozy Hoodie");
        assert_eq!(prod2.handle, "cozy-hoodie");
        assert_eq!(prod2.price_cents, 4999);
        assert_eq!(prod2.inventory_quantity, 20);
        assert_eq!(prod2.published, false);

        // Verify catalog list contains both products
        let products = catalog.list_products();
        assert_eq!(products.len(), 2);
        assert_eq!(products[0], prod1);
        assert_eq!(products[1], prod2);
    }
}
