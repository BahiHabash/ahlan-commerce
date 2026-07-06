/// Cache key for a rendered storefront product page.
///
/// Format: `storefront:product-page:{handle}`
///
/// - `storefront` identifies the read surface.
/// - `product-page` identifies the cached artifact.
/// - `{handle}` is the product handle from the storefront route.
pub fn storefront_product_page(handle: &str) -> String {
    format!("storefront:product-page:{handle}")
}
