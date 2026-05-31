# SDD Integration

This folder tracks optional SDD/Kiro integration state.

`cc-sdd` is the preferred installer/updater. Do not manually fork large SDD skill sets into this template unless there is a deliberate local customization.

## Controlled installation

Use:

```bash
./scripts/install-sdd.sh --dry-run-only --targets=claude
./scripts/install-sdd.sh --targets=claude
```

By default, the installer runs `cc-sdd` in a temporary staging directory, copies generated skills/settings into controlled locations, and stores the generated `CLAUDE.md` memory document at:

```text
.agents/vendor/cc-sdd/CLAUDE.md
```

The live `CLAUDE.md -> .agents/AGENTS.md` symlink is preserved.

Use direct mode only when you explicitly want `cc-sdd` to write live tool files:

```bash
./scripts/install-sdd.sh --mode=direct --memory-policy=replace --targets=claude
```
