# Root task runner for the lotus-explore-rs repository.
# Run `just --list` to see available recipes.

# ── Workspace gate (mirrors .github/workflows/ci.yml) ─────────────────────────

fmt:
	cargo fmt --all -- --check

check:
	cargo check --workspace --all-targets --locked

clippy:
	cargo clippy --workspace --all-targets --locked -- -D warnings

test:
	cargo test --workspace --all-targets --locked --quiet

doc:
	cargo doc --workspace --no-deps --locked

# ── Full CI gate (every check the pipeline runs, in order) ────────────────────
# `just ci`. Each step reuses a recipe above (single source of truth). Supply-chain
# tools that may be absent locally are skipped by their own recipes.

ci:
	just fmt
	just check
	just clippy
	just test
	just doc
	just wasm
	just clippy-wasm
	just machete
	just audit
	just deny

# One `cargo check -p <app>` per app keeps the wasm build green.
wasm:
	cargo check -p lotus-explore-rs --target wasm32-unknown-unknown --features dioxus/wasm-split --locked

# Per-package WASM clippy (NOT `--workspace --target wasm32`: `lotus-deploy`
# is a host-only bin — `reqwest::blocking` cannot exist on wasm — so a
# workspace-wide wasm lint can never pass; only wasm-relevant crates are
# linted here). Mirrors the Clippy-WASM step in .github/workflows/ci.yml.
clippy-wasm:
	cargo clippy --target wasm32-unknown-unknown -p lotus -p lotus-explore-rs --locked -- -D warnings

# ── Per-app dev servers / production builds ───────────────────────────────────
# fetch-ketcher must run from the app crate dir so the relative
# `public/assets/ketcher` default lands inside apps/<app>/public,
# not at the repo-root public/.

serve app:
	cd apps/{{app}} && cargo run -p lotus-deploy --bin fetch-ketcher
	dx serve --package {{app}} --wasm-split --features dioxus/wasm-split

build app:
	cd apps/{{app}} && cargo run -p lotus-deploy --bin fetch-ketcher
	dx build --release --package {{app}} --wasm-split --features dioxus/wasm-split --locked --debug-symbols=false --rustc-args=-Copt-level=z

# ── Supply-chain hygiene (skip gracefully if a tool is not installed) ─────────

machete:
	@command -v cargo-machete >/dev/null 2>&1 && cargo machete || echo "cargo-machete not installed; skipping"

audit:
	@command -v cargo-audit >/dev/null 2>&1 && cargo audit || echo "cargo-audit not installed; skipping"

deny:
	@command -v cargo-deny >/dev/null 2>&1 && cargo deny check advisories bans licenses sources || echo "cargo-deny not installed; skipping"

outdated:
	@command -v cargo-outdated >/dev/null 2>&1 && cargo outdated --workspace --exit-code 1 || echo "cargo-outdated not installed; skipping"

# README sync: regenerate each crate README from README.tpl + source `//!`
# doc comments, lint, and diff against the checked-in README.md.
readme:
	@command -v cargo-readme >/dev/null 2>&1 || { echo "cargo-readme not installed; skipping"; exit 0; }
	@command -v panache >/dev/null 2>&1 || { echo "panache not installed; skipping"; exit 0; }
	@for d in crates/lotus/; do \
	(cd $$d && cargo readme -t README.tpl -o /tmp/readme_panache.md 2>/dev/null && panache lint /tmp/readme_panache.md && diff -q /tmp/readme_panache.md README.md > /dev/null 2>&1 || { echo "README.md out of date for $$d — run: (cd $$d && cargo readme -t README.tpl -o README.md)"; exit 1; }) || exit 1; \
	done
