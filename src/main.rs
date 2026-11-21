use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use anyhow::Context;
use axum_server::tls_rustls::RustlsConfig;
use product_browsing::catalog::Catalog;
use product_browsing::web::{AppState, router};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let database_url = env::var("DATABASE_URL")
        .context("DATABASE_URL is required; it is supplied by the postgres service")?;

    let max_connections = env::var("DATABASE_MAX_CONNECTIONS")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(10);

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(15))
        .connect(&database_url)
        .await
        .context("could not connect to the product catalog database")?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("could not apply the product catalog migrations")?;

    let catalog = Catalog::new(pool);
    let app = router(AppState::new(catalog));

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let address: SocketAddr = format!("{host}:{port}")
        .parse()
        .with_context(|| format!("invalid server address {host}:{port}"))?;

    let cert_path = env::var("TLS_CERT_PATH").unwrap_or_else(|_| ".plot/tls/cert.pem".to_string());
    let key_path = env::var("TLS_KEY_PATH").unwrap_or_else(|_| ".plot/tls/key.pem".to_string());

    let tls = RustlsConfig::from_pem_file(&cert_path, &key_path)
        .await
        .with_context(|| {
            format!("could not load the TLS certificate at {cert_path} and key at {key_path}")
        })?;

    tracing::info!(address = %address, "Product browsing is listening over https");

    axum_server::bind_rustls(address, tls)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .context("the product browsing server stopped")?;

    Ok(())
}
