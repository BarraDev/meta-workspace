## Summary

<!-- One sentence describing what this PR does. -->

## Motivation

<!-- Why is this change needed? Link to an issue if one exists (Closes #___). -->

## Changes

<!-- List the main changes in this PR. -->

- 

## Testing

<!-- How was this tested? Check all that apply. -->

- [ ] Unit tests added or updated (`#[cfg(test)]` in `src/**`)
- [ ] Integration tests added or updated (`tests/cli.rs`)
- [ ] Manually tested with `mw init` + `mw doctor` in a temp directory
- [ ] Template smoke-tested (if `template/` changed)
- [ ] Policy paths tested (allow / warn / deny) if policy changed

## Gates

Before submitting, confirm all of these pass:

```bash
# Run from the repository root (the crate manifest lives there).
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `cargo test` passes (unit + integration + doc)

## Docs updated

- [ ] `tooling/README.md` command table (if a command was added or changed)
- [ ] Root `README.md` command table (if a command was added or changed)
- [ ] `docs/workspace-contract.md` (if the contract changed)
- [ ] `docs/distribution.md` (if the release process changed)

## Notes for reviewers

<!-- Anything specific the reviewer should pay attention to. -->
