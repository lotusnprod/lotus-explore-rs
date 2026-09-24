# lotus-explore-rs

[![AGPL-3.0
license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![Tests](https://img.shields.io/badge/tests-315-brightgreen)](https://github.com/lotusnprod/lotus-explore-rs/actions)

`lotus-explore-rs` --- LOTUS Knowledge Explorer.

A linked open data (LOAD) explorer for the LOTUS compound-taxon-reference
knowledge graph from Wikidata, queried via SPARQL. Powered by the `lotus` shared
crate and the QLever SPARQL endpoint.

## Quick start

```bash
dx serve --package lotus-explore-rs
```

To also run the optional API:

```bash
cargo run --locked --features server -p lotus-explore-rs
```

Then open `http://localhost:8080/?api_base=http://127.0.0.1:8787`.

Without the server, the explorer falls back to direct QLever/SPARQL queries.

## Running the full stack (API + WASM client) in Docker

The included [`Dockerfile`](../../Dockerfile) builds both the API server and the
WASM web bundle (including Ketcher) in a multi-stage build. The runtime image
serves static files via the built-in `ServeDir` fallback.

```bash
# Build the image
docker build -t lotus-explore-rs .

# Run (serves API on :8787 and static files at /app/public)
docker run -p 8080:8787 lotus-explore-rs

# Open http://localhost:8080
```

Environment variables:

  | Variable               | Default       | Description                                                          |
  | ---------------------- | ------------- | -------------------------------------------------------------------- |
  | `HOST`                 | `0.0.0.0`     | Bind address                                                         |
  | `PORT`                 | `8787`        | Listen port                                                          |
  | `PUBLIC_DIR`           | `/app/public` | Static files directory                                               |
  | `LOTUS_API_BASE`       | *(none)*      | Upstream SPARQL/API base URL                                         |
  | `APP_ENV`              | `development` | Use `production` with `CORS_ALLOWED_ORIGINS`                         |
  | `CORS_ALLOWED_ORIGINS` | *(none)*      | Comma-separated allowed origins (required when `APP_ENV=production`) |

## Architecture

See [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md) for the full architectural
overview.

## Development testing

Run the workspace test suite:

```bash
cargo test --workspace --all-targets --locked
```

## Setup: external assets

RDKit.js and citation-js are loaded from CDN (no local download needed). All
document `<head>` metadata, scripts, and styles are managed in Rust via
`ui::document::DocumentHead` --- see `src/document_head.rs`.

### CSS Build Dependencies (Tailwind)

The project uses Tailwind CSS for styling. Install build dependencies:

```bash
npm install
npm run build:css  # Build CSS once, or run 'npm run watch:css' during development
```

The `Dioxus.toml` pre-build hook automatically runs `build:css` during
`dx serve` and `dx build`, so manual CSS rebuilds are only needed for local
development outside the Dioxus build pipeline.

### Ketcher (115 MB)

Ketcher must be fetched before serving or deploying:

```bash
cd apps/lotus-explore-rs   # from repo root
cargo run -p lotus-deploy --bin fetch-ketcher
```

Or simply use the `just` recipes, which fetch Ketcher automatically:

```bash
just build lotus-explore-rs   # fetches Ketcher + dx build --release
just serve lotus-explore-rs   # fetches Ketcher + dx serve
```

## Citation

- Paper (DOI): <https://doi.org/10.7554/eLife.70780>
- BibTeX: [`public/docs/references.bib`](./public/docs/references.bib)

## Site metadata

`public/llms.txt`, `public/humans.txt`, `public/robots.txt`,
`public/.well-known/security.txt`, `public/_headers`, and
`public/site.webmanifest` are generated from
[`metadata/site-metadata.json`](./metadata/site-metadata.json).

## Explorer ⇄ API integration

  | Scenario                | `api_base` source                     | API used            |
  | ----------------------- | ------------------------------------- | ------------------- |
  | Codeberg Pages (public) | none                                  | ✗ direct SPARQL     |
  | Local dev               | auto-detected `http://127.0.0.1:8787` | ✓ if server running |
  | Build-time              | `LOTUS_API_BASE` env var              | ✓                   |
  | Runtime override        | `?api_base=…` query param             | ✓                   |

## URL automation

URL-driven execution and exports:

- `?execute=true` --- run query on load
- `?download=true&format=csv` --- download CSV
- `?download=true&format=json` --- download SPARQL Results JSON
- `?download=true&format=rdf` --- download RDF (Turtle)

When both `download` and `execute` are present, `download` takes priority.

## Archive

A frozen version is archived on Zenodo: <https://doi.org/10.5281/zenodo.5794106>

## License

`AGPL-3.0-only` --- see [`LICENSE`](https://www.gnu.org/licenses/agpl-3.0.html)
for details.
