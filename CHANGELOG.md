# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- `mw policy check`: `draft_only_pr.require_explicit_user_approval: false` now
  loosens the draft-only PR gate (approval not required → allow) instead of
  denying unconditionally; the default (`true`) is unchanged. Added a
  regression test for the opt-out branch.
- `mw sdd` no longer panics on the unreachable pre-1970 clock branch when
  building its staging temp dir (`SystemTime` error now falls back to `0`).

### Changed

- Documentation: removed the `modify{input}` policy decision from the README,
  workspace contract, and module docs — the engine implements only
  `allow`/`deny`/`warn`, so the contract no longer advertises an unimplemented
  variant.

## [0.1.0] - 2026-05-31

First tagged release of the `meta-workspace` project and the `mw` maintenance CLI.

### Added

- `mw` commands: `init`, `doctor`, `links`, `add-project`, `memory`,
  `sdd install/update/status`, `hook session-start`, `policy check`, `migrate`.
- Embedded deployable workspace template materialized by `mw init` (no network,
  no runtime dependency).
- Cross-harness policy engine (`mw policy check`): protected-path denial,
  worktree enforcement, and draft-only PR gating, with Claude (PreToolUse hook)
  and Pi (extension) adapters and Codex/Gemini advisory references.
- Tagged binary releases for Linux, macOS, and Windows via
  `.github/workflows/release.yml`, with SHA-256 checksums.
- CI: fmt + clippy + tests on Linux/macOS/Windows, an MSRV (1.85) job, and a
  `cargo-deny` supply-chain check.

### Changed

- The crate manifest now lives at the repository root so the embedded
  `template/` tree ships inside the package; `mw` is installable via
  `cargo install --git` / `--path` and the crate is publishable to crates.io.
  Rust sources remain under `tooling/`.
- Draft-only PR approval is read only from the `MW_USER_APPROVED` environment
  variable (out-of-band), never from the agent-controlled tool event payload,
  closing a self-approval loophole.

### Fixed

- Windows compatibility-link creation uses real `symlink_dir`/`symlink_file`
  instead of a file copy that failed on the five directory links.
- `mw policy check` no longer false-matches paths that merely contain a
  configured root name as an interior segment.
- Executable lookup (`which`) probes `PATHEXT` on Windows so tools installed as
  `npx.cmd` are found.

[Unreleased]: https://github.com/BarraDev/meta-workspace/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/BarraDev/meta-workspace/releases/tag/v0.1.0
