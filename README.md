# meta-workspace

A reusable **one-company-at-a-time meta-workspace** for coordinating repositories, worktrees, agent instructions, memory, specifications, PR workflows, and project knowledge — without turning every application repository into a pile of agent-specific files.

This repository is the **development project** for the template and its maintenance CLI. It is not itself a deployed workspace. The deployable workspace lives in [`template/`](template/), and the Rust maintenance tool lives in [`tooling/`](tooling/).

## Why this exists

Modern engineering work often spans many repositories and many AI coding tools. Each tool wants its own instruction files, skills, commands, hooks, memory setup, and workflow conventions. If those files are copied into every application repository, they drift quickly and become hard to maintain.

`meta-workspace` solves that by creating a single coordination workspace for one company at a time:

- Application repositories stay clean.
- Shared agent instructions live in one canonical place.
- Claude, Pi, Codex, Gemini, and similar tools get compatibility layers.
- Specs, decisions, memory, PR work, and worktree workflows are organized outside the application code.
- Optional systems such as MemPalace, Prism, and SDD/Kiro can be enabled without becoming base requirements.

The goal is not to replace a monorepo. The goal is to give humans and agents a stable operating base around a company’s repositories.

## Who this is for

This project can help if you:

- work across several repositories for the same company or client;
- use multiple AI coding agents and want one shared instruction layer;
- prefer feature work and reviews in Git worktrees instead of dirtying main checkouts;
- want project memory and specifications without embedding them in application repos;
- need a repeatable template for new companies, clients, or consulting engagements;
- want a zero-runtime base workspace, with optional memory/spec tooling only when selected.

It is especially useful for teams or solo operators who want AI agents to behave consistently across projects while keeping application repositories focused on application code.

## The core idea

There are two separate layers:

| Layer | Location | Purpose |
|---|---|---|
| Content layer | [`template/`](template/) | The deployable workspace files: `.agents/`, `workspace.yaml`, project registry, company profile, agent compatibility links, optional SDD/Kiro folders, and docs. |
| Tooling layer | [`tooling/`](tooling/) | The Rust crate `meta-workspace`, which installs the `mw` binary used to create, repair, validate, and maintain a workspace. |

The content layer has no runtime dependency. The tooling layer is a single Rust binary used for maintenance.

## Why a separate workspace instead of files in every repo?

Because company-level operating rules are not the same as repository-level code.

A meta-workspace keeps these concerns separate:

- **Company context**: profile, terminology, project registry, decisions.
- **Agent context**: shared instructions, skills, commands, policy adapters.
- **Workflow context**: worktrees, PR review rules, handoffs, specs.
- **Optional memory/spec systems**: MemPalace, Prism, SDD/Kiro.

Application repositories can still have their own `AGENTS.md` or tool-specific instructions when needed, but the reusable baseline lives here.

## Repository layout

| Path | Role |
|---|---|
| [`tooling/`](tooling/) | Rust crate `meta-workspace`; installs the `mw` binary. |
| [`template/`](template/) | The embedded deployable workspace that `mw init` materializes. |
| [`docs/`](docs/) | Engineering docs, including the workspace contract and distribution notes. |
| [`.github/`](.github/) | CI for formatting, clippy, and tests. |

The repository root intentionally has no `workspace.yaml`. That prevents the development project from being confused with a deployed company workspace.

## What `mw` does

`mw` is the maintenance CLI for the workspace.

Implemented commands include:

- `mw init` — materialize or repair a workspace from the embedded template.
- `mw doctor` — validate schema, required files, compatibility links, and workspace directories.
- `mw links` — create or repair Claude/Pi/Gemini/agent compatibility links.
- `mw add-project` — add a project to `projects/registry.yaml` with line-based YAML edits.
- `mw memory` — configure optional memory profiles and `.env.local` values.
- `mw sdd install/update/status` — controlled SDD/Kiro integration through `cc-sdd`.
- `mw hook session-start` — non-blocking session warm-up for optional memory systems.
- `mw policy check` — shared policy engine for harness adapters.
- `mw migrate` — schema migration entry point for future versions.

There is intentionally no `mw eject` yet. Eject remains backlog until there is real demand.

## Agent compatibility model

`.agents/` is the canonical support folder. Tool-specific paths are compatibility layers.

Examples:

- `AGENTS.md -> .agents/AGENTS.md`
- `CLAUDE.md -> .agents/AGENTS.md`
- `GEMINI.md -> .agents/AGENTS.md`
- `.claude/skills -> ../.agents/skills`
- `.pi/skills -> ../.agents/skills`

This avoids duplicating instructions across tools.

## Policy model

The workspace has a harness-neutral policy file at:

```text
.agents/policies.yaml
```

`mw policy check` reads tool events as JSON and returns a decision as JSON:

- `allow`
- `deny { reason }`
- `warn { message }`
- `modify { input }`

Claude and Pi can enforce these decisions programmatically. Codex and Gemini currently consume the same policy as advisory guidance because their local hook capabilities are more limited.

## Dependency philosophy

The base workspace should be boring and dependable.

- Base template: no required runtime.
- `mw`: single Rust binary.
- MemPalace: optional Python-based memory system.
- Prism: optional Node/MCP memory system.
- SDD/Kiro: optional Node/`cc-sdd` integration.

This keeps new workspaces usable even before any optional tooling is installed.

## Development workflow

```bash
cd tooling
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Exercise the embedded template:

```bash
cd tooling
cargo build
MW=$PWD/target/debug/mw
D=$(mktemp -d)
(cd "$D" && "$MW" init --company-id demo && "$MW" doctor)
rm -rf "$D"
```

Development follows TDD: write or extend a failing test first, then implement.

## Current status

Completed:

- Phase 1: workspace contract and schema version.
- Phase 2: Rust `mw` crate scaffold, CLI surface, CI, and core commands.
- Phase 3: full parity with retired interim bash/python scripts.
- Phase 3 verification: fixed `mw links` so it reconciles all compatibility links, not only top-level agent files.
- Phase 4 started: policy file, `mw policy check` policy loading, Claude PreToolUse hook, Pi extension adapter, and Codex/Gemini advisory references.

Remaining:

- complete release automation;
- decide when/if to publish the crate to crates.io;
- harden policy evaluation beyond protected paths;
- improve docs and examples for real users;
- review installation, update, and troubleshooting flows.
