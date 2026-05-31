# Contributing to meta-workspace

Thank you for your interest in contributing. This guide covers the development workflow, conventions, and expectations for changes to this repository.

## Repository layout

| Path | Purpose |
|---|---|
| `Cargo.toml` | Crate manifest `meta-workspace` at the repo root; target paths point at `tooling/src`, and the root location lets the embedded `template/` ship in the package. |
| `tooling/` | Rust sources for the `mw` binary (`tooling/src`, `tooling/tests`). |
| `template/` | Embedded deployable workspace that `mw init` materializes. |
| `docs/` | Engineering documentation. |
| `.github/` | CI and community files. |

There is a `Cargo.toml` but no `workspace.yaml` at the repository root. The development project is intentionally separate from a deployed company workspace. Run all `cargo` commands from the repository root.

## Prerequisites

- Rust toolchain (stable). See `rust-toolchain.toml` for the pinned channel; the minimum supported version is in `Cargo.toml` (`rust-version`).
- `cargo` available on `PATH`.
- Git.

Node and Python are not required for the base build. They are only needed when testing `mw sdd` flows that invoke `npx cc-sdd`.

## Development workflow

All changes must pass the standard gate before being committed (run from the repository root):

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Fix formatting automatically:

```bash
cargo fmt
```

### Running against the embedded template

```bash
cargo build
MW=$PWD/target/debug/mw
D=$(mktemp -d)
(cd "$D" && "$MW" init --company-id test && "$MW" doctor)
rm -rf "$D"
```

## Test conventions

This project uses all four Rust test layers:

| Layer | Location | What it covers |
|---|---|---|
| Unit (white-box) | `#[cfg(test)]` in `tooling/src/**` | parsing helpers, policy engine, line-based YAML edits |
| CLI definition | `tooling/src/cli.rs` debug assert | clap wiring is valid |
| Integration (black-box) | `tooling/tests/cli.rs` | binary driven via `std::process::Command` against a temp fixture |
| Doc tests | `///` examples in `tooling/src/workspace.rs` | public helpers stay correct |

**Write a failing test first, then implement.** TDD is the development convention for this project.

Do not add test dependencies (`assert_cmd`, `predicates`, etc.). The integration tests use only the standard library.

## Commits

- Write commits in English.
- Use short imperative subject lines, for example: `Fix doctor missing symlink check`.
- Group related changes into one commit rather than many small fixups.
- Reference an issue number in the subject or body when one exists.

## Pull requests

- One concern per pull request.
- All CI gates must pass before requesting review.
- Prefer small, focused changes over large refactors.
- Document any new `mw` commands or flags in `tooling/README.md` and the commands table in the root `README.md`.
- If you change the workspace contract, update `docs/workspace-contract.md`.

## Template changes

Changes to `template/` are embedded into the binary at build time via `include_dir!`. When modifying template files:

1. Edit the file under `template/`.
2. Rebuild the binary from the repository root: `cargo build`.
3. Verify: `mw init` in a temp directory, then `mw doctor`.

## Adding a new `mw` command

1. Add a new variant to the `Command` enum in `tooling/src/cli.rs`.
2. Add the handler file under `tooling/src/commands/`.
3. Wire it in `tooling/src/main.rs`.
4. Add unit tests in the new module.
5. Add integration coverage in `tooling/tests/cli.rs`.
6. Update command tables in `tooling/README.md` and the root `README.md`.

## Policy changes

Changes to `template/.agents/policies.yaml` or the policy engine in `tooling/src/commands/policy.rs` must be tested with all three decision paths (allow, warn, deny). The integration tests in `tests/cli.rs` include policy scenarios; extend them for new rules.

## Licensing

By contributing you agree that your contributions are licensed under the same `MIT OR Apache-2.0` terms as the project. See `LICENSE-MIT` and `LICENSE-APACHE`.
