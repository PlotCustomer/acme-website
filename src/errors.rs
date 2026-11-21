use askama::Template;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use serde_json::json;

use crate::browse_products::BrowseError;
use crate::catalog::CatalogError;
use crate::domain::UnknownProductType;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    NotFound(String),
    #[error(transparent)]
    Internal(CatalogError),
}

impl From<BrowseError> for ApiError {
    fn from(error: BrowseError) -> Self {
        match error {
            BrowseError::SearchTooLong
            | BrowseError::UnknownCategory(_)
            | BrowseError::UnknownSort(_) => ApiError::BadRequest(error.to_string()),
            BrowseError::Catalog(catalog_error) => ApiError::Internal(catalog_error),
        }
    }
}

impl From<UnknownProductType> for ApiError {
    fn from(_: UnknownProductType) -> Self {
        ApiError::BadRequest("that is not a product category".into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ApiError::BadRequest(message) => (StatusCode::BAD_REQUEST, "bad_request", message),
            ApiError::NotFound(message) => (StatusCode::NOT_FOUND, "not_found", message),
            ApiError::Internal(error) => {
                tracing::error!(error = %error, "Products API request failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "the catalog could not be read".to_string(),
                )
            }
        };

        (
            status,
            Json(json!({
                "error": {
                    "code": code,
                    "message": message,
                }
            })),
        )
            .into_response()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WebError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    NotFound(String),
    #[error(transparent)]
    Internal(CatalogError),
    #[error("template rendering failed: {0}")]
    Render(String),
}

impl From<BrowseError> for WebError {
    fn from(error: BrowseError) -> Self {
        match error {
            BrowseError::SearchTooLong
            | BrowseError::UnknownCategory(_)
            | BrowseError::UnknownSort(_) => WebError::BadRequest(error.to_string()),
            BrowseError::Catalog(catalog_error) => WebError::Internal(catalog_error),
        }
    }
}

impl From<CatalogError> for WebError {
    fn from(error: CatalogError) -> Self {
        WebError::Internal(error)
    }
}

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorPage {
    status: u16,
    title: &'static str,
    message: String,
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        let (status, title, message) = match self {
            WebError::BadRequest(message) => (
                StatusCode::BAD_REQUEST,
                "That request needs adjusting",
                message,
            ),
            WebError::NotFound(message) => (StatusCode::NOT_FOUND, "Product not found", message),
            WebError::Internal(error) => {
                tracing::error!(error = %error, "product browser request failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "The catalog is unavailable",
                    "We could not load the product catalog just now. Please try again.".into(),
                )
            }
            WebError::Render(error) => {
                tracing::error!(error = %error, "page could not be rendered");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "The catalog is unavailable",
                    "We could not render this page just now. Please try again.".into(),
                )
            }
        };

        let page = ErrorPage {
            status: status.as_u16(),
            title,
            message,
        };

        match page.render() {
            Ok(html) => (status, Html(html)).into_response(),
            Err(error) => {
                tracing::error!(error = %error, "error page could not be rendered");
                (status, "The catalog is unavailable").into_response()
            }
        }
    }
}
