use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::browse_products::BrowseView;
use crate::catalog::{Category, Product};
use crate::domain::ProductType;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryValue {
    pub slug: &'static str,
    pub name: &'static str,
}

impl From<ProductType> for CategoryValue {
    fn from(product_type: ProductType) -> Self {
        Self {
            slug: product_type.slug(),
            name: product_type.label(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductResponse {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub summary: String,
    pub description: String,
    pub category: CategoryValue,
    pub price_cents: i64,
    pub currency: String,
    pub price: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Product> for ProductResponse {
    fn from(product: Product) -> Self {
        let price = product.formatted_price();
        Self {
            id: product.id,
            slug: product.slug,
            name: product.name,
            summary: product.summary,
            description: product.description,
            category: CategoryValue::from(product.product_type),
            price_cents: product.price_cents,
            currency: product.currency,
            price,
            created_at: product.created_at,
            updated_at: product.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductsResponse {
    pub products: Vec<ProductResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub query: String,
    pub category: Option<&'static str>,
    pub sort: &'static str,
}

impl From<BrowseView> for ProductsResponse {
    fn from(view: BrowseView) -> Self {
        Self {
            products: view
                .products
                .into_iter()
                .map(ProductResponse::from)
                .collect(),
            total: view.total,
            limit: view.limit,
            offset: view.offset,
            query: view.query,
            category: view.active_category.map(ProductType::slug),
            sort: view.sort.slug(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryResponse {
    pub slug: &'static str,
    pub name: &'static str,
    pub count: i64,
}

impl From<Category> for CategoryResponse {
    fn from(category: Category) -> Self {
        Self {
            slug: category.slug,
            name: category.name,
            count: category.count,
        }
    }
}
