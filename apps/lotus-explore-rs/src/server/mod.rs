// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Native HTTP API for the LOTUS explorer: search and export endpoints wired
//! to an Axum [`Router`].  Migrated from the former `lotus-api` crate so the
//! full native + WASM stack builds from a single package.
//!
//! # Run locally
//!
//! ```bash
//! cargo run --features server -p lotus-explore-rs -- --help                       # list flags; each falls back to $VAR
//! cargo run --features server -p lotus-explore-rs -- --host 0.0.0.0 --port 8787
//! LOTUS_API_BASE=http://localhost:3030 cargo run --features server -p lotus-explore-rs
//! ```
//!
//! # Endpoints
//!
//! - `GET /health`
//! - `GET /metrics`
//! - `POST /v1/search`
//! - `POST /v1/export-url`
//! - `GET /v1/export-file/{cache_key}/{format}`
//! - `GET /openapi.json`
//! - `GET /docs`
//!
//! # Environment variables
//!
//! - `LOTUS_API_BASE` — base URL for the API server
//! - `HOST` — bind address (default: `127.0.0.1`)
//! - `PORT` — bind port (default: `8787`)

mod config;
mod errors;
mod handlers;
mod query_logic;
mod services;
mod state;
mod types;

#[cfg(test)]
mod tests;

use axum::{
    Router,
    body::Body,
    extract::DefaultBodyLimit,
    http::{HeaderName, HeaderValue, header},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
};
use handlers::{export_file, export_urls, health, metrics, search};
use tower_http::{
    compression::CompressionLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::server::{
    config::{AppConfig, build_cors_layer},
    errors::ErrorResponse,
    state::AppState,
    types::{
        ApiElementState, ApiSmilesSearchType, ExportUrlResponse, HealthResponse, RowDto,
        SearchRequest, SearchResponse, SearchStats,
    },
};

const X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::health,
        handlers::metrics,
        handlers::search,
        handlers::export_urls,
        handlers::export_file
    ),
    components(
        schemas(
            HealthResponse,
            SearchRequest,
            SearchResponse,
            SearchStats,
            RowDto,
            ExportUrlResponse,
            ErrorResponse,
            ApiSmilesSearchType,
            ApiElementState
        )
    ),
    tags((name = "lotus-explore-rs", description = "LOTUS explorer programmatic API"))
)]
struct ApiDoc;

pub fn build_router(max_body_bytes: usize, config: &AppConfig, state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics))
        .route("/v1/search", post(search))
        .route("/v1/export-url", post(export_urls))
        .route("/v1/export-file/{cache_key}/{format}", get(export_file))
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
        .with_state(state)
        .layer(DefaultBodyLimit::max(max_body_bytes))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::DEBUG))
                .on_response(DefaultOnResponse::new().level(Level::INFO))
                .on_failure(DefaultOnFailure::new().level(Level::WARN)),
        )
        .layer(PropagateRequestIdLayer::new(X_REQUEST_ID))
        .layer(SetRequestIdLayer::new(X_REQUEST_ID, MakeRequestUuid))
        .layer(middleware::from_fn(add_security_headers))
        .layer(CompressionLayer::new())
        .layer(build_cors_layer(config))
}

async fn add_security_headers(req: axum::http::Request<Body>, next: Next) -> Response {
    let mut response = next.run(req).await;
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    response.headers_mut().insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    response
}

/// Entry point for the in-package native API server.
///
/// Started from [`fn main`](crate::main) when the `server` feature is enabled
/// on a non-WASM target. Mirrors the former standalone `lotus-api` binary so the
/// full native + WASM stack builds and runs from this single package.
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    let config = config::AppConfig::from_env().map_err(std::io::Error::other)?;
    let state = state::AppState::new(&config);
    let app = build_router(config.max_body_bytes, &config, state);

    let addr = config.bind_addr().map_err(std::io::Error::other)?;
    log::info!(
        "lotus-explore-rs server listening on http://{addr} timeout_ms={} max_concurrency={} max_body_bytes={}",
        config.request_timeout.as_millis(),
        config.max_concurrency,
        config.max_body_bytes,
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut signal) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            let _ = signal.recv().await;
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => { log::info!("event=shutdown signal=ctrl_c"); },
        () = terminate => { log::info!("event=shutdown signal=terminate"); },
    }
}
