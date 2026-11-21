use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::{Json, Router};
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::api_response::{CategoryResponse, ProductResponse, ProductsResponse};
use crate::browse_products::{BrowseQuery, BrowseView, DEFAULT_PAGE_SIZE};
use crate::catalog::{Category, Product, ProductSort};
use crate::domain::ProductType;
use crate::errors::{ApiError, WebError};
use crate::product_browser::ProductBrowser;
use crate::products_api::ProductsApi;

#[derive(Clone)]
pub struct AppState {
    pub browser: ProductBrowser,
    pub products_api: ProductsApi,
}

impl AppState {
    pub fn new(catalog: crate::catalog::Catalog) -> Self {
        let products_api = ProductsApi::new(catalog);
        Self {
            browser: ProductBrowser::new(products_api.clone()),
            products_api,
        }
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(|| async { Redirect::permanent("/products") }))
        .route("/products", get(browse_page))
        .route("/products/{slug}", get(product_page))
        .route("/api/health", get(health))
        .route("/api/products", get(api_products))
        .route("/api/products/categories", get(api_categories))
        .route("/api/products/{slug}", get(api_product))
        .nest_service("/static", ServeDir::new("static"))
        .fallback(fallback)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

struct CategoryLink {
    name: &'static str,
    count: i64,
    href: String,
    active: bool,
}

struct SortLink {
    name: &'static str,
    href: String,
    active: bool,
}

#[derive(Template)]
#[template(path = "products.html")]
struct ProductsPage {
    products: Vec<Product>,
    total: i64,
    query: String,
    active_category_name: Option<&'static str>,
    active_category_slug: Option<&'static str>,
    sort_slug: &'static str,
    category_links: Vec<CategoryLink>,
    sort_links: Vec<SortLink>,
    page_start: i64,
    page_end: i64,
    previous_href: Option<String>,
    next_href: Option<String>,
    clear_href: String,
    has_filters: bool,
}

#[derive(Template)]
#[template(path = "product.html")]
struct ProductPage {
    product: Product,
    category_href: String,
}

async fn browse_page(
    State(state): State<AppState>,
    Query(query): Query<BrowseQuery>,
) -> Result<Html<String>, WebError> {
    let view = state.browser.browse(query).await?;
    let categories = state.browser.categories().await?;
    let page = build_products_page(view, categories);
    render(page)
}

async fn product_page(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Html<String>, WebError> {
    let product =
        state.browser.product(&slug).await?.ok_or_else(|| {
            WebError::NotFound(format!("No product is listed at /products/{slug}."))
        })?;

    let category_href = products_url(
        "",
        Some(product.product_type),
        ProductSort::Name,
        0,
        DEFAULT_PAGE_SIZE,
    );
    render(ProductPage {
        product,
        category_href,
    })
}

async fn api_products(
    State(state): State<AppState>,
    Query(query): Query<BrowseQuery>,
) -> Result<Json<ProductsResponse>, ApiError> {
    let view = state.browser.browse(query).await?;
    Ok(Json(ProductsResponse::from(view)))
}

async fn api_categories(
    State(state): State<AppState>,
) -> Result<Json<Vec<CategoryResponse>>, ApiError> {
    let categories = state.browser.categories().await?;
    Ok(Json(
        categories.into_iter().map(CategoryResponse::from).collect(),
    ))
}

async fn api_product(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<ProductResponse>, ApiError> {
    let product = state
        .products_api
        .get(&slug)
        .await
        .map_err(ApiError::Internal)?
        .ok_or_else(|| {
            ApiError::NotFound(format!("No product is listed at /api/products/{slug}."))
        })?;

    Ok(Json(ProductResponse::from(product)))
}

async fn health(State(state): State<AppState>) -> Response {
    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(state.products_api.pool())
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "ok", "database": "ok"})),
        )
            .into_response(),
        Err(error) => {
            tracing::error!(error = %error, "health check could not reach the database");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({"status": "degraded", "database": "unavailable"})),
            )
                .into_response()
        }
    }
}

async fn fallback() -> WebError {
    WebError::NotFound("The page you requested does not exist.".into())
}

fn render<T: Template>(page: T) -> Result<Html<String>, WebError> {
    page.render()
        .map(Html)
        .map_err(|error| WebError::Render(error.to_string()))
}

fn build_products_page(view: BrowseView, categories: Vec<Category>) -> ProductsPage {
    let all_count = categories.iter().map(|category| category.count).sum();
    let active_slug = view.active_category.map(ProductType::slug);

    let mut category_links = Vec::with_capacity(categories.len() + 1);
    category_links.push(CategoryLink {
        name: "All products",
        count: all_count,
        href: products_url(&view.query, None, view.sort, 0, view.limit),
        active: view.active_category.is_none(),
    });
    category_links.extend(categories.into_iter().map(|category| CategoryLink {
        name: category.name,
        count: category.count,
        href: products_url(
            &view.query,
            ProductType::from_slug(category.slug),
            view.sort,
            0,
            view.limit,
        ),
        active: active_slug == Some(category.slug),
    }));

    let sort_links = ProductSort::ALL
        .into_iter()
        .map(|sort| SortLink {
            name: sort.label(),
            href: products_url(&view.query, view.active_category, sort, 0, view.limit),
            active: view.sort == sort,
        })
        .collect();

    let page_start = if view.products.is_empty() {
        0
    } else {
        view.offset + 1
    };
    let page_end = view.offset + view.products.len() as i64;
    let previous_href = (view.offset > 0).then(|| {
        products_url(
            &view.query,
            view.active_category,
            view.sort,
            (view.offset - view.limit).max(0),
            view.limit,
        )
    });
    let next_href = (page_end < view.total).then(|| {
        products_url(
            &view.query,
            view.active_category,
            view.sort,
            view.offset + view.limit,
            view.limit,
        )
    });

    ProductsPage {
        products: view.products,
        total: view.total,
        has_filters: !view.query.is_empty()
            || view.active_category.is_some()
            || view.sort != ProductSort::Name,
        active_category_name: view.active_category.map(ProductType::label),
        active_category_slug: view.active_category.map(ProductType::slug),
        sort_slug: view.sort.slug(),
        clear_href: "/products".to_string(),
        query: view.query,
        category_links,
        sort_links,
        page_start,
        page_end,
        previous_href,
        next_href,
    }
}

fn products_url(
    query: &str,
    product_type: Option<ProductType>,
    sort: ProductSort,
    offset: i64,
    limit: i64,
) -> String {
    let mut params = Vec::new();

    if !query.trim().is_empty() {
        params.push(format!("q={}", urlencoding::encode(query.trim())));
    }
    if let Some(product_type) = product_type {
        params.push(format!("category={}", product_type.slug()));
    }
    if sort != ProductSort::Name {
        params.push(format!("sort={}", sort.slug()));
    }
    if offset > 0 {
        params.push(format!("offset={offset}"));
    }
    if limit != DEFAULT_PAGE_SIZE {
        params.push(format!("limit={limit}"));
    }

    if params.is_empty() {
        "/products".to_string()
    } else {
        format!("/products?{}", params.join("&"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_urls_keep_the_active_filters() {
        let url = products_url(
            "trail gear",
            Some(ProductType::EverydayGoods),
            ProductSort::PriceLowToHigh,
            12,
            DEFAULT_PAGE_SIZE,
        );

        assert_eq!(
            url,
            "/products?q=trail%20gear&category=everyday-goods&sort=price-asc&offset=12"
        );
    }

    #[test]
    fn the_default_view_has_a_clean_url() {
        assert_eq!(
            products_url("", None, ProductSort::Name, 0, DEFAULT_PAGE_SIZE),
            "/products"
        );
    }
}
