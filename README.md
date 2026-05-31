# meta-workspace

This repository is the **development project** for the generic, one-company-at-a-time
meta-workspace template and its maintenance tool. It is **not** itself a deployed
workspace — the deployable workspace lives under [`template/`](template/).

## Layout

| Path | Role |
|------|------|
| [`tooling/`](tooling/) | Rust crate `meta-workspace` → the `mw` binary (the maintenance tool). |
| [`template/`](template/) | The deployable workspace content (`.agents/`, `workspace.yaml`, scripts, agent-compat symlinks). What `mw init` materializes. |
| [`docs/`](docs/) | Project engineering docs: the [workspace contract](docs/workspace-contract.md) and [distribution](docs/distribution.md). |
| `.github/` | CI for the tooling. |

The repo root is intentionally **not** a workspace: there is no `workspace.yaml`
at the root, so `mw` only resolves a workspace when run inside `template/` (or a
real deployed instance). This keeps the project cleanly separated from an
instance of the thing it produces.

## Two layers (recap)

- **Content layer** — the files in `template/`. Requires no runtime; a user can
  copy/clone it and use it with any agent harness.
- **Tooling layer** — the `mw` binary in `tooling/`. Needed only to *create or
  maintain* a workspace, never to *use* one.

## Working on the project

```bash
# Tooling (Rust)
cd tooling
cargo test                                # unit + integration + doc tests
cargo clippy --all-targets -- -D warnings
cargo fmt --check

# Inspect the deployable template as a workspace
cd ../template
../tooling/target/debug/mw doctor
```

Development follows TDD: write or extend a failing test first, then implement.
See [`docs/workspace-contract.md`](docs/workspace-contract.md) for the full
contract the tooling implements.

## Status

- Phase 1: workspace contract + `schemaVersion`. Done.
- Phase 2: `mw` crate scaffolded (clap), functional `doctor`/`memory`/`hook`/`policy`/`migrate`, full test matrix. Done.
- Phase 3: porting the stub commands to real behavior, TDD. Done — `add-project`, `links`, `init`, and `sdd install/update`. Remaining: retire the interim bash+python scripts as each reaches parity.
- Phase 4: `cargo-dist` release pipeline and crate publication. Planned.
