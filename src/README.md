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

## Architecture

See [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md) for the full architectural
overview.

## Engineering skills

- [`SKILLS.md`](./SKILLS.md)
- [`docs/skills/SUGGESTIONS.md`](./docs/skills/SUGGESTIONS.md)

## Curation share links

- [`docs/CURATION_SHARE_LINKS.md`](./docs/CURATION_SHARE_LINKS.md)

## Development testing

Run logging format tests during telemetry work:

```bash
cargo test --locked -p lotus-explore-rs utils::logging::tests
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

For detailed guidance on using Tailwind with Dioxus, see
[`TAILWIND_MIGRATION.md`](./TAILWIND_MIGRATION.md).

### Ketcher (115 MB)

Ketcher must be fetched before serving or deploying:

```bash
cargo run --release -p lotus-deploy --bin fetch-ketcher
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
