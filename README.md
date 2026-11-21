# Product browsing

A Rust/Axum delivery of the **Product browsing** work package: the ACME Product
Catalog browser and the Products API it reads through.

## What is here

- **Product Browser** (`src/product_browser.rs`) — the application function. It
  realizes the Product Catalog application service and reads the persisted
  `Product` data object through the catalog.
- **Browse products** (`src/browse_products.rs`) — the application process. It
  validates search, category, sort and pagination input before reading.
- **Products API** (`/api/products`) — the application interface. It is served
  from the same Axum application and reads the same catalog.
- **Product Catalog** (`src/catalog.rs`) — the catalog read model over the
  PostgreSQL `products` table.
- **Website** — the Axum application, its HTML views, CSS, and the API routes.

The catalog is organised around the three modelled product families:

- Everyday Goods (`everyday-goods`)
- Explosives & Weapons (`explosives-weapons`)
- Vehicles & Gadgets (`vehicles-gadgets`)

## Run

The server reads `DATABASE_URL` from the environment and listens on HTTPS.
`scripts/start.sh` creates a self-signed development certificate on first run if
`TLS_CERT_PATH` / `TLS_KEY_PATH` are not supplied.

```sh
export DATABASE_URL='postgresql://postgres@127.0.0.1:5432/product_catalog'
./scripts/start.sh
```

The migrations in `migrations/` run on startup and seed the catalog.

Environment settings:

| Setting | Purpose | Default |
| --- | --- | --- |
| `DATABASE_URL` | PostgreSQL connection for the catalog | required |
| `HOST` | Interface the server binds to | `0.0.0.0` |
| `PORT` | HTTPS port | `3000` |
| `TLS_CERT_PATH` | PEM certificate | `.plot/tls/cert.pem` |
| `TLS_KEY_PATH` | PEM private key | `.plot/tls/key.pem` |
| `DATABASE_MAX_CONNECTIONS` | Pool size | `10` |
| `RUST_LOG` | Log filter | `info` |

## Pages

- `GET /` — redirects to the catalog.
- `GET /products` — product grid with search, category filters, sort, and
  pagination.
- `GET /products/{slug}` — product detail.

## Products API

`GET /api/products` accepts:

- `q` — text search across name, summary and description.
- `category` — `everyday-goods`, `explosives-weapons`, or `vehicles-gadgets`.
- `sort` — `name` (default), `price-asc`, or `price-desc`.
- `limit` — page size, 1–48, default 12.
- `offset` — zero-based page offset.

It returns `products`, `total`, `limit`, `offset`, `query`, `category`, and
`sort`. Each product carries its category, price in cents, formatted price, and
timestamps.

Other endpoints:

- `GET /api/products/{slug}` — one product.
- `GET /api/products/categories` — category names and product counts.
- `GET /api/health` — database readiness.

## Checks

```sh
cargo fmt --all -- --check
cargo check --all-targets
cargo test
```
