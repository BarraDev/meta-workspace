# Memory Workflow

Memory is optional and configurable per workspace.

Supported profiles:

- `none`: no memory integration.
- `mempalace`: use MemPalace CLI.
- `prism`: use Prism MCP.
- `full`: use MemPalace and Prism when both are available.

Rules:

1. Load configured memory at session start.
2. Search memory before decisions that may depend on past context.
3. Store durable corrections, decisions, and handoffs when useful.
4. Never store secrets.
5. Never trust memory blindly; validate against live state.
6. Do not mine large directories without explicit scope.
