#!/usr/bin/env bash
# Universal non-blocking session-start hook for AI agent runtimes.

set +e

ROOT="${CLAUDE_PROJECT_DIR:-$(pwd)}"
ENV_FILE="$ROOT/.env.local"

if [ -f "$ENV_FILE" ]; then
  # shellcheck disable=SC1090
  . "$ENV_FILE"
fi

WING="${MEMPALACE_WING:-}"
if [ -z "$WING" ]; then
  WING="$(basename "$ROOT")"
fi

if ! command -v mempalace >/dev/null 2>&1; then
  echo "[session-start] mempalace CLI not found; skipping warm-up" >&2
  exit 0
fi

mempalace status >/dev/null 2>&1
status_rc=$?
mempalace wake-up --wing "$WING" >/dev/null 2>&1
wake_rc=$?

if [ $status_rc -eq 0 ] && [ $wake_rc -eq 0 ]; then
  echo "[session-start] mempalace ready (wing=$WING)" >&2
else
  echo "[session-start] mempalace warm-up degraded (status=$status_rc wake=$wake_rc); continuing" >&2
fi

exit 0
