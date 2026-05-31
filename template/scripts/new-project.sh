#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

PROJECT_ID=""
PROJECT_NAME=""
REPO_URL=""
DEFAULT_BRANCH=""
LANGUAGE=""
INSTRUCTIONS=""
NON_INTERACTIVE=false

usage() {
  cat <<'EOF'
Usage: ./scripts/new-project.sh [options]

Options:
  --id=<slug>                 Project id. Required in --non-interactive mode.
  --name=<display-name>       Project display name.
  --repo-url=<url>            Repository URL.
  --default-branch=<branch>   Default branch. Default: main.
  --language=<value>          Primary language/tooling.
  --instructions=<path>       Project instructions path.
  --non-interactive           Do not prompt; fail if required values are missing.
  -h, --help                  Show this help.

Application code remains outside this meta-workspace by default:
  - clean checkout: ../repos/<project-id>
  - worktrees:      ../worktrees
EOF
}

for arg in "$@"; do
  case "$arg" in
    --id=*) PROJECT_ID="${arg#--id=}" ;;
    --name=*) PROJECT_NAME="${arg#--name=}" ;;
    --repo-url=*) REPO_URL="${arg#--repo-url=}" ;;
    --default-branch=*) DEFAULT_BRANCH="${arg#--default-branch=}" ;;
    --language=*) LANGUAGE="${arg#--language=}" ;;
    --instructions=*) INSTRUCTIONS="${arg#--instructions=}" ;;
    --non-interactive) NON_INTERACTIVE=true ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unsupported argument: $arg"; usage; exit 1 ;;
  esac
done

if [ -z "$PROJECT_ID" ]; then
  if [ "$NON_INTERACTIVE" = true ]; then
    echo "--id is required with --non-interactive"
    exit 1
  fi
  read -r -p "Project id (slug, required): " PROJECT_ID
fi
PROJECT_ID="$(printf '%s' "$PROJECT_ID" | tr '[:upper:]' '[:lower:]' | tr -cs 'a-z0-9._-' '-' | sed 's/^-//; s/-$//')"
if [ -z "$PROJECT_ID" ]; then
  echo "Project id is required."
  exit 1
fi

if [ -z "$PROJECT_NAME" ]; then
  if [ "$NON_INTERACTIVE" = true ]; then
    PROJECT_NAME="$PROJECT_ID"
  else
    read -r -p "Project display name [$PROJECT_ID]: " PROJECT_NAME
    PROJECT_NAME="${PROJECT_NAME:-$PROJECT_ID}"
  fi
fi

if [ -z "$REPO_URL" ] && [ "$NON_INTERACTIVE" != true ]; then
  read -r -p "Repository URL (optional): " REPO_URL
fi

if [ -z "$DEFAULT_BRANCH" ]; then
  if [ "$NON_INTERACTIVE" = true ]; then
    DEFAULT_BRANCH="main"
  else
    read -r -p "Default branch [main]: " DEFAULT_BRANCH
    DEFAULT_BRANCH="${DEFAULT_BRANCH:-main}"
  fi
fi

if [ -z "$LANGUAGE" ] && [ "$NON_INTERACTIVE" != true ]; then
  read -r -p "Primary language/tooling (optional): " LANGUAGE
fi

if [ -z "$INSTRUCTIONS" ]; then
  if [ "$NON_INTERACTIVE" = true ]; then
    INSTRUCTIONS="docs/instructions/$PROJECT_ID.md"
  else
    read -r -p "Project instructions path [docs/instructions/$PROJECT_ID.md]: " INSTRUCTIONS
    INSTRUCTIONS="${INSTRUCTIONS:-docs/instructions/$PROJECT_ID.md}"
  fi
fi

PROJECT_ID="$PROJECT_ID" PROJECT_NAME="$PROJECT_NAME" REPO_URL="$REPO_URL" DEFAULT_BRANCH="$DEFAULT_BRANCH" LANGUAGE="$LANGUAGE" INSTRUCTIONS="$INSTRUCTIONS" python3 - <<'PY'
import json
import os
from pathlib import Path

registry = Path('projects/registry.yaml')
text = registry.read_text() if registry.exists() else 'projects: []\n'
project_id = os.environ['PROJECT_ID']

for line in text.splitlines():
    stripped = line.strip()
    if stripped.startswith('- id:'):
        existing = stripped.removeprefix('- id:').strip()
        try:
            existing = json.loads(existing)
        except Exception:
            existing = existing.strip('"\'')
        if existing == project_id:
            raise SystemExit(f'Project already appears to exist in projects/registry.yaml: {project_id}')

def yaml_value(value):
    return 'null' if value == '' else json.dumps(value)

entry = f'''  - id: {yaml_value(project_id)}
    name: {yaml_value(os.environ['PROJECT_NAME'])}
    repository:
      url: {yaml_value(os.environ['REPO_URL'])}
      main_path: {yaml_value(f'../repos/{project_id}')}
      default_branch: {yaml_value(os.environ['DEFAULT_BRANCH'])}
    worktrees:
      root: "../worktrees"
      naming: {yaml_value(f'{project_id}-{{task-or-branch}}')}
    language: {yaml_value(os.environ['LANGUAGE'])}
    instructions: {yaml_value(os.environ['INSTRUCTIONS'])}
'''

if text.startswith('projects: []'):
    text = text.replace('projects: []', 'projects:\n' + entry.rstrip('\n'), 1)
elif text.startswith('projects:\n'):
    marker = '\n\n# Example:'
    if marker in text:
        text = text.replace(marker, '\n' + entry.rstrip('\n') + marker, 1)
    else:
        if not text.endswith('\n'):
            text += '\n'
        text += entry
else:
    raise SystemExit('Unsupported registry format; edit projects/registry.yaml manually.')

registry.write_text(text if text.endswith('\n') else text + '\n')
PY

mkdir -p "$(dirname "$INSTRUCTIONS")"
if [ ! -f "$INSTRUCTIONS" ]; then
  cat > "$INSTRUCTIONS" <<EOF
# $PROJECT_NAME Instructions

Fill in project-specific instructions before editing this repository.

- Repository: ../repos/$PROJECT_ID
- Worktrees: ../worktrees
- Default branch: $DEFAULT_BRANCH
EOF
  echo "created $INSTRUCTIONS"
fi

echo "added project $PROJECT_ID to projects/registry.yaml"
echo "clean checkout path: ../repos/$PROJECT_ID"
echo "worktree root: ../worktrees"
