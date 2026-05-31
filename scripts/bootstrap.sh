#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

DISPLAY_NAME="${WORKSPACE_NAME:-}"
SLUG="${WORKSPACE_SLUG:-}"
INIT_GIT=""
CREATE_DIRS=""
NON_INTERACTIVE=false

usage() {
  cat <<'EOF'
Usage: ./scripts/bootstrap.sh [options]

Options:
  --name=<display-name>       Workspace/company display name.
  --slug=<slug>               Workspace slug.
  --init-git=<yes|no>         Initialize git if needed.
  --create-dirs=<yes|no>      Create ../repos ../worktrees ../scratch ../archives ../logs.
  --non-interactive           Use defaults for missing values.
  -h, --help                  Show this help.
EOF
}

for arg in "$@"; do
  case "$arg" in
    --name=*) DISPLAY_NAME="${arg#--name=}" ;;
    --slug=*) SLUG="${arg#--slug=}" ;;
    --init-git=*) INIT_GIT="${arg#--init-git=}" ;;
    --create-dirs=*) CREATE_DIRS="${arg#--create-dirs=}" ;;
    --non-interactive) NON_INTERACTIVE=true ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unsupported argument: $arg"; usage; exit 1 ;;
  esac
done

normalize_yes_no() {
  local value="$1"
  case "$(printf '%s' "$value" | tr '[:upper:]' '[:lower:]')" in
    y|yes|true|1) echo "Y" ;;
    n|no|false|0) echo "N" ;;
    *) echo "$value" ;;
  esac
}

echo "Meta-workspace bootstrap"
echo

if [ -z "$DISPLAY_NAME" ]; then
  if [ "$NON_INTERACTIVE" = true ]; then
    DISPLAY_NAME="Meta Workspace"
  else
    read -r -p "Workspace/company display name [Meta Workspace]: " DISPLAY_NAME
    DISPLAY_NAME="${DISPLAY_NAME:-Meta Workspace}"
  fi
fi

if [ -z "$SLUG" ]; then
  if [ "$NON_INTERACTIVE" = true ]; then
    SLUG="meta-workspace"
  else
    read -r -p "Workspace slug [meta-workspace]: " SLUG
    SLUG="${SLUG:-meta-workspace}"
  fi
fi

if [ -z "$INIT_GIT" ]; then
  if [ "$NON_INTERACTIVE" = true ]; then
    INIT_GIT="Y"
  else
    read -r -p "Initialize git repository if needed? [Y/n]: " INIT_GIT
    INIT_GIT="${INIT_GIT:-Y}"
  fi
fi
INIT_GIT="$(normalize_yes_no "$INIT_GIT")"

if [ -z "$CREATE_DIRS" ]; then
  if [ "$NON_INTERACTIVE" = true ]; then
    CREATE_DIRS="Y"
  else
    read -r -p "Create parent folders ../repos ../worktrees ../scratch ../archives ../logs? [Y/n]: " CREATE_DIRS
    CREATE_DIRS="${CREATE_DIRS:-Y}"
  fi
fi
CREATE_DIRS="$(normalize_yes_no "$CREATE_DIRS")"

if [[ "$CREATE_DIRS" =~ ^[Yy]$ ]]; then
  mkdir -p ../repos ../worktrees ../scratch ../archives ../logs
  echo "created/verified parent folders"
fi

./scripts/install-agent-links.sh

if [[ "$INIT_GIT" =~ ^[Yy]$ ]] && [ ! -d .git ]; then
  git init
  echo "initialized git repository"
fi

if [ ! -f .env.local ]; then
  cat > .env.local <<EOF
MEMPALACE_WING=$SLUG
PRISM_PROJECT=$SLUG
MEMORY_PROFILE=none
EOF
  echo "created .env.local"
else
  echo "kept existing .env.local"
fi

DISPLAY_NAME="$DISPLAY_NAME" SLUG="$SLUG" python3 - <<'PY' 2>/dev/null || true
import os
import re
from pathlib import Path

name = os.environ['DISPLAY_NAME']
slug = os.environ['SLUG']

profile = Path('company/profile.md')
if profile.exists():
    text = profile.read_text()
    text = re.sub(r'^- Name:.*$', f'- Name: {name}', text, count=1, flags=re.MULTILINE)
    text = re.sub(r'^- Slug:.*$', f'- Slug: {slug}', text, count=1, flags=re.MULTILINE)
    profile.write_text(text)

workspace = Path('workspace.yaml')
if workspace.exists():
    text = workspace.read_text()
    text = re.sub(r'^(  name: ).*$', rf'\1{slug}', text, count=1, flags=re.MULTILINE)
    text = re.sub(r'^(  company_id: ).*$', rf'\1{slug}', text, count=1, flags=re.MULTILINE)
    text = re.sub(r'^(  company_name: ).*$', rf'\1{name}', text, count=1, flags=re.MULTILINE)
    workspace.write_text(text)
PY

echo
echo "Bootstrap complete. Next steps:"
echo "  1. Edit company/profile.md and projects/registry.yaml"
echo "  2. Optional memory setup: ./scripts/install-memory.sh"
echo "  3. Optional SDD setup: ./scripts/install-sdd.sh"
echo "  4. Validate: ./scripts/doctor.sh"
