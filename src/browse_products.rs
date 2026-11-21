use serde::Deserialize;

use crate::catalog::{CatalogError, Category, Product, ProductFilter, ProductSort};
use crate::domain::ProductType;
use crate::products_api::ProductsApi;

pub const DEFAULT_PAGE_SIZE: i64 = 12;

/// The query a visitor sends while browsing the catalog.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct BrowseQuery {
    pub q: Option<String>,
    pub category: Option<String>,
    pub sort: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct BrowseView {
    pub products: Vec<Product>,
    pub total: i64,
    pub query: String,
    pub active_category: Option<ProductType>,
    pub sort: ProductSort,
    pub limit: i64,
    pub offset: i64,
}

/// The `Browse products` application process. It validates the visitor's
/// request and turns it into a catalog read.
#[derive(Debug, Clone)]
pub struct BrowseProducts {
    products_api: ProductsApi,
}

impl BrowseProducts {
    pub fn new(products_api: ProductsApi) -> Self {
        Self { products_api }
    }

    pub async fn run(&self, query: BrowseQuery) -> Result<BrowseView, BrowseError> {
        let query_text = query.q.unwrap_or_default().trim().to_string();
        if query_text.chars().count() > 120 {
            return Err(BrowseError::SearchTooLong);
        }

        let active_category = match query.category.as_deref().map(str::trim) {
            None | Some("") | Some("all") => None,
            Some(value) => Some(
                ProductType::from_slug(value)
                    .ok_or_else(|| BrowseError::UnknownCategory(value.to_string()))?,
            ),
        };

        let sort = match query.sort.as_deref().map(str::trim) {
            None | Some("") => ProductSort::Name,
            Some(value) => ProductSort::parse(value)
                .ok_or_else(|| BrowseError::UnknownSort(value.to_string()))?,
        };

        let limit = query.limit.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, 48);
        let offset = query.offset.unwrap_or(0).max(0);

        let filter = ProductFilter {
            query: (!query_text.is_empty()).then_some(query_text.clone()),
            product_type: active_category,
            sort,
            limit,
            offset,
        };

        let page = self.products_api.list(&filter).await?;

        Ok(BrowseView {
            products: page.products,
            total: page.total,
            query: query_text,
            active_category,
            sort,
            limit,
            offset,
        })
    }

    pub async fn categories(&self) -> Result<Vec<Category>, BrowseError> {
        Ok(self.products_api.categories().await?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BrowseError {
    #[error("search terms are limited to 120 characters")]
    SearchTooLong,
    #[error("unknown product category: {0}")]
    UnknownCategory(String),
    #[error("unknown product sort: {0}")]
    UnknownSort(String),
    #[error(transparent)]
    Catalog(#[from] CatalogError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_query_longer_than_the_limit_is_rejected_before_the_read() {
        let error = BrowseError::SearchTooLong;
        assert_eq!(
            error.to_string(),
            "search terms are limited to 120 characters"
        );
    }
}
