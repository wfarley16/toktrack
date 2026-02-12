#!/bin/bash
# Auto-populate session metadata sidecar on first user prompt
# Hook type: UserPromptSubmit
# Creates ~/.toktrack/sessions/<session-id>.json with git branch + issue ID
set -e
INPUT=$(cat)
SESSION_ID=$(echo "$INPUT" | jq -r '.session_id // ""')

if [ -z "$SESSION_ID" ] || [ "$SESSION_ID" = "" ]; then
  exit 0
fi

SIDECAR_DIR="$HOME/.toktrack/sessions"
SIDECAR_FILE="$SIDECAR_DIR/$SESSION_ID.json"

# Write session ID to tmp for /track skill
echo "$SESSION_ID" > /tmp/toktrack-session-id

# Idempotent: skip if sidecar already exists
if [ -f "$SIDECAR_FILE" ]; then
  exit 0
fi

mkdir -p "$SIDECAR_DIR"

# Get git branch
GIT_BRANCH=""
if command -v git &>/dev/null; then
  GIT_BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "")
fi

# Extract issue ID from branch (e.g., ISE-123 from feature/ISE-123-foo)
ISSUE_ID=""
if [ -n "$GIT_BRANCH" ]; then
  ISSUE_ID=$(echo "$GIT_BRANCH" | grep -oE '[A-Z]+-[0-9]+' | head -1 || echo "")
fi

NOW=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Build sidecar JSON
jq -n \
  --arg sid "$SESSION_ID" \
  --arg issue "$ISSUE_ID" \
  --arg branch "$GIT_BRANCH" \
  --arg now "$NOW" \
  '{
    session_id: $sid,
    tags: [],
    skills_used: [],
    auto_detected: {
      git_branch: (if $branch != "" then $branch else null end),
      issue_id_source: (if $issue != "" then "branch" else null end)
    },
    created_at: $now,
    updated_at: $now
  } + (if $issue != "" then {issue_id: $issue} else {} end)' > "$SIDECAR_FILE"

exit 0
