# lotus-explore-rs Architecture

## Source layout

```
src/
  app/          — application bootstrap, shell, and root view
  components/   — rendering-only UI components
  features/
    explore/    — search, results, download, error recovery
    curation/   — data curation workflows
  hooks/        — Dioxus reactive hooks
  services/     — cross-feature application services
  state/        — shared application state
  api/          — API client, DTOs, config, error types
  repositories/ — data access (maps DTOs to domain types)
  models/       — shared domain types
  core/         — shared error and primitive types
  utils/        — pure helpers
  i18n/         — locale strings
  ui/           — accessibility contracts and smoke checks
  download/     — download effects (wasm + native)
  export/       — export metadata and filename resolution
```

## Rules

- Components render and dispatch --- no business logic.
- Feature internals are private; each feature exposes a typed facade via
  `mod.rs`.
- API DTOs stop at repository boundaries and never reach components.
- State subscriptions are narrow --- components read only the slices they use.
- Every async path has a stable token; stale completions are discarded before
  state commit.
- Typed errors (`thiserror`) at all boundaries; user messages are derived
  separately.

## Data flow

```
api client → dto mapper → repository → service → state controller → component
```

## Agent tooling

- `.github/ai/` --- AI collaboration guides, the contribution protocol, and the
  incident postmortem for the lotus-explore Qlever 429-storm fix.
