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
	just machete
	just audit
	just deny

# One `cargo check -p <app>` per app keeps the wasm build green.
wasm:
	cargo check -p lotus-explore-rs --target wasm32-unknown-unknown --locked

# ── Per-app dev servers / production builds ───────────────────────────────────

serve app:
	dx serve --package {{app}}

build app:
	dx build --release --package {{app}}

# ── Supply-chain hygiene (skip gracefully if a tool is not installed) ─────────

machete:
	@command -v cargo-machete >/dev/null 2>&1 && cargo machete check --workspace || echo "cargo-machete not installed; skipping"

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
