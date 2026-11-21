#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if ! command -v cargo >/dev/null 2>&1; then
    if command -v mise >/dev/null 2>&1; then
        eval "$(mise activate bash)"
    fi
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "cargo is required to start the product browsing server" >&2
    exit 1
fi

TLS_CERT_PATH="${TLS_CERT_PATH:-$ROOT/.plot/tls/cert.pem}"
TLS_KEY_PATH="${TLS_KEY_PATH:-$ROOT/.plot/tls/key.pem}"

if [[ ! -f "$TLS_CERT_PATH" || ! -f "$TLS_KEY_PATH" ]]; then
    if ! command -v openssl >/dev/null 2>&1; then
        echo "openssl is required to create the development TLS certificate" >&2
        exit 1
    fi

    mkdir -p "$(dirname "$TLS_CERT_PATH")" "$(dirname "$TLS_KEY_PATH")"
    SAN="DNS:localhost,IP:127.0.0.1"
    if [[ -n "${TLS_HOSTNAME:-}" ]]; then
        SAN="$SAN,DNS:$TLS_HOSTNAME"
    fi

    openssl req -x509 -newkey rsa:2048 -sha256 -days 3650 -nodes \
        -keyout "$TLS_KEY_PATH" \
        -out "$TLS_CERT_PATH" \
        -subj "/CN=ACME Product Catalog" \
        -addext "subjectAltName=$SAN" >/dev/null 2>&1
fi

export TLS_CERT_PATH
export TLS_KEY_PATH

exec cargo run --release --manifest-path "$ROOT/Cargo.toml"
