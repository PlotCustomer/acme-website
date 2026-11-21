use sqlx::PgPool;

use crate::catalog::{Catalog, CatalogError, Category, Product, ProductFilter, ProductPage};

/// The Products API application interface.
///
/// It is the contract the Product Browser reads through, and the same reads are
/// published by the `/api/products` routes.
#[derive(Debug, Clone)]
pub struct ProductsApi {
    catalog: Catalog,
}

impl ProductsApi {
    pub fn new(catalog: Catalog) -> Self {
        Self { catalog }
    }

    pub fn pool(&self) -> &PgPool {
        self.catalog.pool()
    }

    pub async fn list(&self, filter: &ProductFilter) -> Result<ProductPage, CatalogError> {
        self.catalog.products(filter).await
    }

    pub async fn get(&self, slug: &str) -> Result<Option<Product>, CatalogError> {
        self.catalog.product(slug).await
    }

    pub async fn categories(&self) -> Result<Vec<Category>, CatalogError> {
        self.catalog.categories().await
    }
}
