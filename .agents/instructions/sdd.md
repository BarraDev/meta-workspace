# SDD / Kiro Workflow

SDD/Kiro support is optional and should be installed or updated with `cc-sdd`.

Reference:

- Repository: https://github.com/gotalab/cc-sdd
- Install command: `npx cc-sdd@latest`

Recommended flow:

```bash
./scripts/install-sdd.sh
```

The installer should run a dry run first and ask for confirmation before applying changes.

Use SDD when work benefits from explicit requirements, design, task boundaries, validation, and resumable implementation. Do not force SDD for small direct changes.
