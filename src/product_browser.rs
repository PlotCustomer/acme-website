use crate::browse_products::{BrowseError, BrowseProducts, BrowseQuery, BrowseView};
use crate::catalog::{CatalogError, Category, Product};
use crate::products_api::ProductsApi;

/// The Product Browser application function.
///
/// It realizes the Product Catalog application service by reading through the
/// Products API, and it composes the `Browse products` application process.
#[derive(Debug, Clone)]
pub struct ProductBrowser {
    products_api: ProductsApi,
    browse_products: BrowseProducts,
}

impl ProductBrowser {
    pub fn new(products_api: ProductsApi) -> Self {
        Self {
            browse_products: BrowseProducts::new(products_api.clone()),
            products_api,
        }
    }

    pub async fn browse(&self, query: BrowseQuery) -> Result<BrowseView, BrowseError> {
        self.browse_products.run(query).await
    }

    pub async fn categories(&self) -> Result<Vec<Category>, BrowseError> {
        self.browse_products.categories().await
    }

    pub async fn product(&self, slug: &str) -> Result<Option<Product>, CatalogError> {
        self.products_api.get(slug).await
    }
}
