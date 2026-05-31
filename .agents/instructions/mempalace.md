# MemPalace

MemPalace support is optional and should use the official Python CLI.

Typical commands:

```bash
mempalace status
mempalace wake-up --wing <workspace-wing>
mempalace search "query" --wing <workspace-wing>
```

Use `./scripts/install-memory.sh` to create a local `.env.local` with a workspace-specific wing.

Do not configure or run MemPalace MCP unless the user explicitly requests it.
