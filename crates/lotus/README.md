# lotus

[![AGPL-3.0
license](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)
[![Tests](https://img.shields.io/badge/tests-84-brightgreen)](https://github.com/lotusnprod/lotus-explore-rs/actions)

## lotus --- LOTUS domain & SPARQL shared core

The single source of truth for everything LOTUS/Wikidata/QLever across the
`lotus-explore-rs` repository. The `lotus-explore-rs` app --- serving both its
native `/v1` API (under `--features server`) and its WASM client --- and any
future app consume this crate rather than constructing queries or parsing CSV
results themselves.

### Module map

  | Module    | Responsibility                                                                                                                                     |
  | --------- | -------------------------------------------------------------------------------------------------------------------------------------------------- |
  | transport | Platform-agnostic SPARQL-over-HTTP: retries, content-negotiation, gateway-error detection. Accepts any endpoint URL.                               |
  | models    | LOTUS domain types: `SearchCriteria`, `CompoundEntry`, `DatasetStats`, `TaxonMatch`, sort state, element constants.                                |
  | queries   | SPARQL query-string builders — `query_all_compounds`, `query_sachem`, `query_with_server_filters`, etc. No I/O.                                    |
  | sparql    | LOTUS-specific wrappers combining transport + models: execute against the default QLever/Wikidata endpoint, parse CSV result sets into typed rows. |

### Design non-goals

- File upload, blob streaming, or progress reporting → see the `upload` crate.
- UI components or styling → see the `ui` crate.
- Application routing, Dioxus state machines, or i18n → these live in each app.

## License

`AGPL-3.0-only` --- see [`LICENSE`](https://www.gnu.org/licenses/agpl-3.0.html)
for details.
