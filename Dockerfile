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
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# --locked ensures Cargo.lock is respected exactly (no silent upgrades)
RUN cargo build --release --locked --features server -p lotus-explore-rs

# ── Stage 2: runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim

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

COPY --from=builder /build/target/release/lotus-explore-rs /usr/local/bin/lotus-explore-rs

# Bind to all interfaces by default when running in a container
ENV HOST=0.0.0.0
ENV PORT=8787

EXPOSE 8787

USER appuser

# Liveness probe — requires the /health endpoint to respond
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider "http://localhost:${PORT}/health" || exit 1

CMD ["lotus-explore-rs"]
