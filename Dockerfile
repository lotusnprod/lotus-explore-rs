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
FROM rust:1.97.0-slim-bookworm AS wasm-builder

WORKDIR /build

# Reuse compiled dependencies from the builder stage (target/ only).
# Each FROM rust:1.97 image has its own cargo/rustup home at /usr/local/.
COPY --from=builder /build/target /build/target

# Install system deps (nodejs for Tailwind, curl for dioxus-cli download)
RUN apt-get update && apt-get install -y --no-install-recommends \
    nodejs npm curl pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install dioxus-cli and the wasm target for building the WASM web bundle.
# Use --target x86_64-unknown-linux-musl to get a statically-linked dx binary
# that doesn't depend on a specific glibc version (bookworm has glibc 2.36,
# but the GNU release needs 2.39).
RUN rustup default 1.97.0 && \
    rustup target add wasm32-unknown-unknown && \
    curl -fsSL -o /tmp/cargo-binstall.tgz \
      https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz && \
    tar -xzf /tmp/cargo-binstall.tgz -C /usr/local/bin/ && \
    chmod +x /usr/local/bin/cargo-binstall && \
    cargo binstall --target x86_64-unknown-linux-musl dioxus-cli --version 0.7.10 --locked --no-confirm

COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY apps/ apps/

# Fetch Ketcher (115 MB) then build the WASM web bundle
RUN cargo run --release -p lotus-deploy --bin fetch-ketcher && \
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
