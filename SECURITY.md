# Security Policy

## Supported versions

This project is in active development. Security fixes are applied to the latest
commit on `main`.

## Reporting a vulnerability

If you believe you have found a security issue in this project, please do **not**
open a public GitHub issue.

Report it privately through the GitHub **Security** tab on this repository:
https://github.com/BarraDev/meta-workspace/security/advisories/new

Include as much detail as possible:

- Description of the issue and potential impact.
- Steps to reproduce or a proof-of-concept.
- Affected component (`mw` binary, template files, policy engine, CI workflow).
- Any mitigations you are aware of.

We aim to acknowledge reports within five business days and provide a remediation
timeline within fourteen.

## Security model

The `mw` binary is a local maintenance CLI. It reads and writes files in the
current workspace directory and its parent sibling directories. It does not open
network connections, bind ports, or transmit data externally.

### Secrets handling

- Secrets and API keys must go in `.env.local`, which is gitignored.
- The deployable workspace template ships `template/.env.example` documenting
  expected variable names and safe placeholder values; `mw init` materializes it
  into a new workspace.
- `mw policy check` explicitly denies writes to `.env`, `.env.*`, and
  `secrets/` by default (see `template/.agents/policies.yaml`).
- Never commit actual credentials to this repository.

### Draft-only PR approval

When the `draft_only_pr` policy is enabled, `mw policy check` blocks PR
comment/review/approval/publish events. Authorization is read **only** from the
`MW_USER_APPROVED` environment variable (`1`/`true`/`yes`), never from the tool
event payload. This is deliberate: the agent that constructs a tool call also
controls the payload JSON, so a payload field could be self-set; the parent
environment of the separate `mw policy check` subprocess cannot. The trade-off
is that approval is coarse and session-scoped rather than per-action — an
accepted limitation. Programmatic enforcement is available on Claude and Pi;
Codex and Gemini consume the policy as advisory guidance only.

### Policy enforcement

`mw policy check` reads harness events from stdin and returns allow/deny/warn
decisions as JSON to stdout. The policy file (`.agents/policies.yaml`) is
workspace-local; it does not fetch rules from a remote source.

### Dependency supply chain

Rust dependencies are declared in the root `Cargo.toml` and pinned via the root
`Cargo.lock`. Dependabot opens PRs when Cargo dependencies or GitHub Actions
receive updates, and CI runs `cargo-deny` (`deny.toml`) to gate advisories,
licenses, and sources.

## Out of scope

- Vulnerabilities in optional third-party tools (MemPalace, Prism, cc-sdd).
  Please report those to the respective upstream projects.
- Issues that require physical access to the developer's machine.
