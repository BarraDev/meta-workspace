# meta-workspace (`mw`)

`mw` is the maintenance CLI (the **tooling layer**) for the generic
one-company-at-a-time meta-workspace template. A workspace's **content layer**
(files, symlinks, git) requires no runtime; `mw` is only needed to create or
maintain a workspace, never to *use* one.

- crate name: `meta-workspace`
- binary name: `mw`
- language: Rust, single static binary, zero runtime dependency
- contract: [`docs/workspace-contract.md`](../docs/workspace-contract.md)

## Build

```bash
cargo build --release   # produces target/release/mw
cargo test
```

## Testing

The crate is split into a thin `mw` binary over a `meta_workspace` library so it
can be tested every way Rust supports:

| Layer | Location | What it covers |
|-------|----------|----------------|
| Unit (white-box) | `#[cfg(test)]` in `src/**` | parsing helpers, policy brain, line-based YAML edits |
| CLI definition | `src/cli.rs` (`debug_assert`) | clap arg/subcommand wiring is valid |
| Integration (black-box) | `tests/cli.rs` | runs the compiled binary against a temp fixture workspace, asserts stdout + exit codes |
| Doc tests | `///` examples in `src/workspace.rs` | documented public helpers stay correct |

Run them all, plus the lint/format gates:

```bash
cargo test                              # unit + integration + doc tests
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

No extra test dependencies are used — the integration tests drive the binary
via `std::process::Command` and `CARGO_BIN_EXE_mw`.

## Commands

| Command | Status | Notes |
|---------|--------|-------|
| `mw init` | working | materialize/repair a workspace from the embedded template |
| `mw doctor` | working | validate the workspace against the contract |
| `mw links` | working | reconcile compat symlinks |
| `mw add-project` | working | append to `projects/registry.yaml` |
| `mw memory` | working | read/set the memory profile (`none\|mempalace\|prism\|full`) + `.env.local` |
| `mw sdd` | working | `status` + controlled cc-sdd `install`/`update` |
| `mw hook session-start` | working | non-blocking mempalace warm-up |
| `mw policy check` | working | cross-harness enforcement engine (stdin event → stdout decision) |
| `mw migrate` | working | upgrade `schemaVersion` (no steps registered for v1) |

`mw eject` is intentionally **not** implemented (backlog, gated on demand).

## Enforcement engine

`mw policy check` reads a Claude-Code-shaped event as JSON on stdin and returns
a decision (`allow` / `deny{reason}` / `warn{message}`) as JSON on stdout. Exit
code mirrors the decision (`0` allow/warn, `1` deny) for shell-only shims.

```bash
echo '{"tool_name":"Write","tool_input":{"file_path":".env"}}' | mw policy check
# {"decision":"deny","reason":"writing to protected path is not allowed: .env"}
```

Real programmatic deny is available on Claude Code and Pi; Codex and Gemini are
advisory. See the contract for adapter details.
