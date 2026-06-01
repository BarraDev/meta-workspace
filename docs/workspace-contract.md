# Workspace Contract

This document defines the stable contract for a generic meta-workspace instance.

Both the future `mw` Rust binary and any harness adapter (or `mw eject` output) must implement and respect this contract. The contract is intentionally runtime-agnostic: the **content layer** (files in a user workspace) requires no runtime; the **tooling layer** (the `mw` binary) maintains it.

## Versioning

- `workspace.yaml` carries `schemaVersion`.
- Current version: `1`.
- The tooling must:
  - read `schemaVersion`,
  - run `mw doctor` validation against it,
  - provide `mw migrate` when a newer tool meets an older workspace.

## Distribution and naming

- Language: Rust (single static binary, zero runtime dependency).
- crate name: `meta-workspace` (crates.io — verified available).
- binary name: `mw`.
- Current Phase 4A distribution: `cargo install --git https://github.com/BarraDev/meta-workspace --locked` (or `--path` from a checkout). Tagged binary releases via `.github/workflows/release.yml`.
- Future official distribution channels (npm-free):
  - GitHub Releases (prebuilt binaries, checksums),
  - `cargo-binstall` (prebuilt fetch),
  - `cargo install meta-workspace` from crates.io, if publishing is approved,
  - `curl | sh` installer + Homebrew tap (later),
  - release pipeline managed by `cargo-dist` on GitHub runners.

## Two layers

- Content layer: `.agents/`, `workspace.yaml`, `company/`, `projects/`, `.kiro/`, `.sdd/`, docs, symlinks. No runtime required.
- Tooling layer: the `mw` binary. Required only to create/maintain a workspace, never to "use" one.

## Dependency tiers

| Tier | Component | Requires | When |
|------|-----------|----------|------|
| 0 | Base workspace (files, symlinks, git) | nothing | always |
| 1 | `mw` maintenance | the `mw` binary | only when running `mw` |
| 2a | Memory: mempalace | python + mempalace CLI | only if user selects mempalace |
| 2b | Memory: prism | node MCP runtime | only if user selects prism |
| 3 | SDD/Kiro | node + cc-sdd | only if user selects SDD |

mempalace and prism are always optional. The base never depends on node or python.

## Commands (tooling contract)

The `mw` binary must provide at least:

- `mw init` — materialize/repair a workspace from the content template.
- `mw doctor` — validate the workspace against this contract.
- `mw links` — create/repair agent compatibility symlinks and adapters.
- `mw add-project` — add an entry to `projects/registry.yaml` (supports non-interactive flags).
- `mw memory` — configure memory profile (`none|mempalace|prism|full`).
- `mw sdd` — controlled cc-sdd install/update (staged by default).
- `mw hook session-start` — optional session warm-up (see below).
- `mw policy check` — policy enforcement engine (see below).
- `mw migrate` — upgrade an older workspace to the current `schemaVersion`.
- `mw eject` — (backlog, not implemented now) emit local fallback scripts.

Interactive prompts must have non-interactive flag equivalents for automation.

## Structured-file editing rule

To stay robust without a heavyweight YAML parser (and to avoid the unmaintained `serde_yaml` crate):

- `mw` owns the shape of `workspace.yaml`, `.sdd/manifest.json`, and `projects/registry.yaml`.
- Editing strategy: token substitution on init, anchored single-line edits for known keys, whole-file writes for JSON, and append-with-duplicate-check for registry entries.
- Prefer regenerate-from-template over deep-merging arbitrary user YAML.
- `JSON` uses `serde_json` (first-class). `YAML` stays token/line-based.

## Session warm-up

Session warm-up is provided by:

```
mw hook session-start
```

(The former interim `scripts/session-start.sh` has been retired now that the
Rust binary is at parity.)

Behavior:

- read memory profile from `workspace.yaml`,
- `none` -> exit 0 immediately (common case),
- `mempalace`/`full` -> mempalace warm-up,
- `prism` -> no warm-up needed (harness starts the MCP server); may print readiness,
- always non-blocking; never fail the session.

## Enforcement engine (the cross-harness "hook equivalent")

### Goal

Provide one enforcement brain that every harness can call through its own native interception mechanism, mirroring how Claude Code hooks shell out to a command.

### Protocol

```
mw policy check
```

- reads an event as JSON on stdin,
- returns a decision as JSON on stdout,
- decision is one of: `allow`, `deny{reason}`, `warn{message}`.

The canonical event JSON shape follows Claude Code's hook payload so Claude needs zero translation; other harnesses translate to/from it in their thin adapter.

### Harness enforcement capability (verified)

| Harness | Native engine | Programmatic deny |
|---------|---------------|-------------------|
| Claude Code | hooks (`PreToolUse`, `SessionStart`, ...) | yes |
| Pi | extensions: `on("tool_call")` returns `{ block: true, reason }`, plus `session_before_*` gates, input/result rewrite | yes |
| Codex | `config.toml` sandbox + approval policy | partial (built-in policy only) |
| Gemini | instructions / MCP | advisory only |

### Adapters (generated by `mw init` / `mw links`)

- Claude -> `.claude/settings.json` hooks -> `command: mw policy check`.
- Pi -> generated `.pi/extensions/mw-policy.ts` that on `tool_call` spawns `mw policy check` and returns `{ block: true, reason }` on deny.
- Codex -> map supported subset to `config.toml`; remainder degrades to instructions.
- Gemini -> `AGENTS.md`/`GEMINI.md` instructions (+ MCP later).

When a harness later gains a real hook engine, only a new adapter generator is added; policy logic never moves.

### Policies (defined once, harness-neutral)

Policies live in a neutral file (`.agents/policies.yaml`) and are evaluated by `mw`:

- protect paths: deny writes to `.env`, `secrets/`.
- enforce worktree: deny/warn write tools targeting configured clean checkout paths such as `../repos`, while allowing configured worktree paths such as `../worktrees`.
- draft-only PR: block PR comment/review/approve/post events unless the user has authorized publishing out-of-band via the `MW_USER_APPROVED` environment variable (`1`/`true`/`yes`). Approval is read from the environment, never from the event payload, so the agent constructing the tool call cannot self-approve.
- session warm-up: only if mempalace/prism configured.

### Honest limits

- True programmatic deny is available only on Claude and Pi.
- Codex and Gemini are advisory (static config + instructions).
- One `mw` spawn per tool call is acceptable for a Rust binary; a long-lived daemon mode can be added later if needed.

## Eject (deferred)

`mw eject` is intentionally not implemented now. It stays in the backlog, gated on real demand. If added, it emits POSIX shell (mac/Linux) generated/embedded by the binary, never a hand-maintained parallel script suite. The contract must remain clean enough that eject can be added later without restructuring.
