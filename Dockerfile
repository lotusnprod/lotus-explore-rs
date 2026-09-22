# ── Stage 1: build ────────────────────────────────────────────────────────────
# Pin to the same channel as rust-toolchain.toml so the build is reproducible.
FROM rust:1.97.0-slim-bookworm AS builder

WORKDIR /build

# Copy manifests first, then ALL app sources: `cargo build -p lotus-explore-rs`
# resolves the entire workspace, so every member's manifest must be present.
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY apps/ apps/

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# --locked ensures Cargo.lock is respected exactly (no silent upgrades)
RUN cargo build --release --locked --features server -p lotus-explore-rs

# ── Stage 2: WASM client (with Ketcher) ──────────────────────────────────────
# Uses ubuntu:24.04 (glibc 2.39) because the pre-built `dx` binary from
# Dioxus releases requires glibc >= 2.39 (Debian Bookworm only has 2.36).
# Rust toolchain is copied from the builder stage (forward-compatible).
FROM ubuntu:24.04 AS wasm-builder

# Reuse the Rust toolchain and compiled deps from the builder stage.
COPY --from=builder /usr/local/rustup /usr/local/rustup
COPY --from=builder /usr/local/cargo /usr/local/cargo
COPY --from=builder /build/target /build/target
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH="/usr/local/cargo/bin:/usr/local/rustup/toolchains/1.97.0-x86_64-unknown-linux-gnu/bin:${PATH}"

WORKDIR /build

# System deps: nodejs/npm (for Tailwind pre-build), curl (for dx download)
RUN apt-get update && apt-get install -y --no-install-recommends \
    nodejs npm curl pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Add the wasm target, then download the pre-built `dx` binary directly
# from the Dioxus GitHub release (GNU variant — works on ubuntu:24.04 glibc 2.39).
# This avoids the slow `cargo install dioxus-cli` from source.
RUN rustup default 1.97.0 && \
    rustup target add wasm32-unknown-unknown && \
    curl -fsSL -o /tmp/dx.tar.gz \
      https://github.com/DioxusLabs/dioxus/releases/download/v0.7.10/dx-x86_64-unknown-linux-gnu.tar.gz && \
    tar -xzf /tmp/dx.tar.gz -C /usr/local/bin/ && \
    chmod +x /usr/local/bin/dx && \
    dx --version

COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY apps/ apps/

# Fetch Ketcher (115 MB) then build the WASM web bundle.
# fetch-ketcher runs from apps/lotus-explore-rs/ so the relative
# `public/assets/ketcher` lands inside the app crate's public/ dir.
RUN cd apps/lotus-explore-rs && \
    cargo run --release -p lotus-deploy --bin fetch-ketcher && \
    dx build --release --platform web --base-path "/" --package lotus-explore-rs

# ── Stage 3: export (for CI artifact extraction) ────────────────────────────────
# Exposes the built web bundle via a scratch image so CI can extract it with
# `docker buildx build --target export --output _final_site .` without needing
# a full runtime stage.
FROM scratch AS export
COPY --from=wasm-builder /build/target/dx/lotus-explore-rs/release/web/public /

# ── Stage 4: runtime ────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

# ── OCI image labels ─────────────────────────────────────────────────────────
LABEL org.opencontainers.image.title="lotus-explore-rs" \
      org.opencontainers.image.description="LOTUS explorer API + client" \
      org.opencontainers.image.url="https://github.com/lotusnprod/lotus-explore-rs" \
      org.opencontainers.image.source="https://github.com/lotusnprod/lotus-explore-rs" \
      org.opencontainers.image.licenses="AGPL-3.0" \
      org.opencontainers.image.vendor="lotusnprod"

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Run as a non-root user — principle of least privilege
RUN adduser --system --no-create-home --uid 1001 appuser

# Copy server binary and WASM web bundle
COPY --from=builder /build/target/release/lotus-explore-rs /usr/local/bin/lotus-explore-rs
COPY --from=wasm-builder /build/target/dx/lotus-explore-rs/release/web/public /app/public

# Bind to all interfaces by default when running in a container
ENV HOST=0.0.0.0
ENV PORT=8787
ENV PUBLIC_DIR=/app/public

EXPOSE 8787

USER appuser

# Liveness probe — requires the /health endpoint to respond
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider "http://localhost:${PORT}/health" || exit 1

CMD ["lotus-explore-rs"]
