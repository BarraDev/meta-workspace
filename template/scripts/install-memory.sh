#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

PROFILE="${MEMORY_PROFILE:-}"
SLUG="${MEMPALACE_WING:-}"
NON_INTERACTIVE=false

usage() {
  cat <<'EOF'
Usage: ./scripts/install-memory.sh [options]

Options:
  --profile=<profile>      none | mempalace | prism | full
  --slug=<slug>            Workspace memory slug.
  --non-interactive        Use defaults for missing values.
  -h, --help               Show this help.
EOF
}

for arg in "$@"; do
  case "$arg" in
    --profile=*) PROFILE="${arg#--profile=}" ;;
    --slug=*) SLUG="${arg#--slug=}" ;;
    --non-interactive) NON_INTERACTIVE=true ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unsupported argument: $arg"; usage; exit 1 ;;
  esac
done

if [ -z "$PROFILE" ]; then
  if [ "$NON_INTERACTIVE" = true ]; then
    PROFILE="mempalace"
  else
    read -r -p "Memory profile (none/mempalace/prism/full) [mempalace]: " PROFILE
    PROFILE="${PROFILE:-mempalace}"
  fi
fi
PROFILE="$(printf '%s' "$PROFILE" | tr '[:upper:]' '[:lower:]' | tr -d '[:space:]')"

case "$PROFILE" in
  none|mempalace|prism|full) ;;
  *) echo "Unsupported memory profile: $PROFILE"; exit 1 ;;
esac

if [ -z "$SLUG" ]; then
  if [ "$NON_INTERACTIVE" = true ]; then
    SLUG="$(basename "$PWD")"
  else
    read -r -p "Workspace memory slug [$(basename "$PWD")]: " SLUG
    SLUG="${SLUG:-$(basename "$PWD")}"
  fi
fi

cat > .env.local <<EOF
MEMPALACE_WING=$SLUG
PRISM_PROJECT=$SLUG
MEMORY_PROFILE=$PROFILE
EOF

echo "wrote .env.local"

case "$PROFILE" in
  mempalace|full)
    if command -v mempalace >/dev/null 2>&1; then
      mempalace status || true
      mempalace wake-up --wing "$SLUG" || true
    else
      echo "mempalace CLI not found. Install/configure it separately if wanted."
    fi
    ;;
esac

case "$PROFILE" in
  prism|full)
    echo "Prism selected. Use .mcp.example.json as a starting point if your agent supports MCP config files."
    ;;
esac

PROFILE="$PROFILE" SLUG="$SLUG" python3 - <<'PY' 2>/dev/null || true
import os
import re
from pathlib import Path

profile = os.environ['PROFILE']
slug = os.environ['SLUG']
text_path = Path('workspace.yaml')
if not text_path.exists():
    raise SystemExit(0)
text = text_path.read_text()
text = re.sub(r'^(  profile: ).*$', rf'\1{profile} # none | mempalace | prism | full', text, count=1, flags=re.MULTILINE)
text = re.sub(r'^(    enabled: ).*$', rf'\1{str(profile in ("mempalace", "full")).lower()}', text, count=1, flags=re.MULTILINE)
text = re.sub(r'^(    wing: ).*$', rf'\1{slug if profile in ("mempalace", "full") else "null"}', text, count=1, flags=re.MULTILINE)
lines = text.splitlines()
in_prism = False
for i, line in enumerate(lines):
    if line == '  prism:':
        in_prism = True
        continue
    if in_prism and line.startswith('  ') and not line.startswith('    '):
        in_prism = False
    if in_prism and line.startswith('    enabled:'):
        lines[i] = f'    enabled: {str(profile in ("prism", "full")).lower()}'
    if in_prism and line.startswith('    project:'):
        lines[i] = f'    project: {slug if profile in ("prism", "full") else "null"}'
text_path.write_text('\n'.join(lines) + '\n')
PY

echo "Memory profile configured as: $PROFILE"
