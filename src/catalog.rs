use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::domain::ProductType;

#[derive(Debug, Clone, FromRow)]
struct ProductRow {
    id: Uuid,
    slug: String,
    name: String,
    summary: String,
    description: String,
    product_type: String,
    price_cents: i64,
    currency: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// A product in the catalog, backed by the persisted `Product` data object.
#[derive(Debug, Clone, Serialize)]
pub struct Product {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub summary: String,
    pub description: String,
    pub product_type: ProductType,
    pub price_cents: i64,
    pub currency: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Product {
    pub fn formatted_price(&self) -> String {
        let amount = self.price_cents as f64 / 100.0;
        match self.currency.as_str() {
            "USD" => format!("${amount:.2}"),
            "EUR" => format!("€{amount:.2}"),
            "GBP" => format!("£{amount:.2}"),
            other => format!("{other} {amount:.2}"),
        }
    }
}

impl TryFrom<ProductRow> for Product {
    type Error = CatalogError;

    fn try_from(row: ProductRow) -> Result<Self, Self::Error> {
        let product_type = ProductType::from_slug(&row.product_type)
            .ok_or_else(|| CatalogError::UnknownProductType(row.product_type.clone()))?;

        Ok(Self {
            id: row.id,
            slug: row.slug,
            name: row.name,
            summary: row.summary,
            description: row.description,
            product_type,
            price_cents: row.price_cents,
            currency: row.currency,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ProductSort {
    #[default]
    Name,
    PriceLowToHigh,
    PriceHighToLow,
}

impl ProductSort {
    pub const ALL: [ProductSort; 3] = [
        ProductSort::Name,
        ProductSort::PriceLowToHigh,
        ProductSort::PriceHighToLow,
    ];

    pub const fn slug(self) -> &'static str {
        match self {
            ProductSort::Name => "name",
            ProductSort::PriceLowToHigh => "price-asc",
            ProductSort::PriceHighToLow => "price-desc",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            ProductSort::Name => "Name A–Z",
            ProductSort::PriceLowToHigh => "Price: low to high",
            ProductSort::PriceHighToLow => "Price: high to low",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "name" | "" => Some(ProductSort::Name),
            "price-asc" => Some(ProductSort::PriceLowToHigh),
            "price-desc" => Some(ProductSort::PriceHighToLow),
            _ => None,
        }
    }

    const fn order_clause(self) -> &'static str {
        match self {
            ProductSort::Name => " ORDER BY name ASC, slug ASC",
            ProductSort::PriceLowToHigh => " ORDER BY price_cents ASC, name ASC",
            ProductSort::PriceHighToLow => " ORDER BY price_cents DESC, name ASC",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProductFilter {
    pub query: Option<String>,
    pub product_type: Option<ProductType>,
    pub sort: ProductSort,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone)]
pub struct ProductPage {
    pub products: Vec<Product>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Category {
    pub slug: &'static str,
    pub name: &'static str,
    pub count: i64,
}

/// The catalog read model. It is the application service the Product Browser
/// realizes, and the Products API exposes the same reads to its consumers.
#[derive(Debug, Clone)]
pub struct Catalog {
    pool: PgPool,
}

impl Catalog {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn products(&self, filter: &ProductFilter) -> Result<ProductPage, CatalogError> {
        let mut products_query: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT id, slug, name, summary, description, product_type, \
             price_cents, currency, created_at, updated_at FROM products",
        );
        push_filters(&mut products_query, filter);
        products_query.push(filter.sort.order_clause());
        products_query
            .push(" LIMIT ")
            .push_bind(filter.limit)
            .push(" OFFSET ")
            .push_bind(filter.offset);

        let rows = products_query
            .build_query_as::<ProductRow>()
            .fetch_all(&self.pool)
            .await?;

        let products = rows
            .into_iter()
            .map(Product::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        let mut count_query: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM products");
        push_filters(&mut count_query, filter);
        let total = count_query
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await?;

        Ok(ProductPage { products, total })
    }

    pub async fn product(&self, slug: &str) -> Result<Option<Product>, CatalogError> {
        let row = sqlx::query_as::<_, ProductRow>(
            "SELECT id, slug, name, summary, description, product_type, \
             price_cents, currency, created_at, updated_at \
             FROM products WHERE slug = $1",
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?;

        row.map(Product::try_from).transpose()
    }

    pub async fn categories(&self) -> Result<Vec<Category>, CatalogError> {
        let rows = sqlx::query_as::<_, (String, i64)>(
            "SELECT product_type, COUNT(*)::bigint FROM products GROUP BY product_type",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut counts = std::collections::HashMap::new();
        for (slug, count) in rows {
            let product_type = ProductType::from_slug(&slug)
                .ok_or_else(|| CatalogError::UnknownProductType(slug.clone()))?;
            counts.insert(product_type, count);
        }

        Ok(ProductType::ALL
            .into_iter()
            .map(|product_type| Category {
                slug: product_type.slug(),
                name: product_type.label(),
                count: counts.get(&product_type).copied().unwrap_or(0),
            })
            .collect())
    }
}

fn push_filters(builder: &mut QueryBuilder<Postgres>, filter: &ProductFilter) {
    builder.push(" WHERE TRUE");

    if let Some(product_type) = filter.product_type {
        builder
            .push(" AND product_type = ")
            .push_bind(product_type.slug().to_string());
    }

    if let Some(query) = filter
        .query
        .as_deref()
        .map(str::trim)
        .filter(|query| !query.is_empty())
    {
        let escaped = query
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        let pattern = format!("%{escaped}%");
        builder
            .push(" AND (name ILIKE ")
            .push_bind(pattern.clone())
            .push(" OR summary ILIKE ")
            .push_bind(pattern.clone())
            .push(" OR description ILIKE ")
            .push_bind(pattern)
            .push(")");
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("catalog contains an unknown product category: {0}")]
    UnknownProductType(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_is_formatted_for_the_catalog() {
        let product = Product {
            id: Uuid::nil(),
            slug: "test-product".into(),
            name: "Test product".into(),
            summary: String::new(),
            description: String::new(),
            product_type: ProductType::EverydayGoods,
            price_cents: 4850,
            currency: "USD".into(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(product.formatted_price(), "$48.50");
    }

    #[test]
    fn sort_slugs_round_trip() {
        for sort in ProductSort::ALL {
            assert_eq!(ProductSort::parse(sort.slug()), Some(sort));
        }
        assert!(ProductSort::parse("newest").is_none());
    }
}
