#!/bin/bash
# Track skill usage in session metadata sidecar
# Hook type: PreToolUse (matcher: Skill)
# Appends skill name to skills_used array if not already present
set -e
INPUT=$(cat)
SKILL_NAME=$(echo "$INPUT" | jq -r '.tool_input.skill // ""')
SESSION_ID=$(echo "$INPUT" | jq -r '.session_id // ""')

if [ -z "$SKILL_NAME" ] || [ -z "$SESSION_ID" ]; then
  exit 0
fi

SIDECAR_DIR="$HOME/.toktrack/sessions"
SIDECAR_FILE="$SIDECAR_DIR/$SESSION_ID.json"

# Skip if sidecar doesn't exist yet (session-metadata.sh creates it)
if [ ! -f "$SIDECAR_FILE" ]; then
  exit 0
fi

# Check if skill already tracked
ALREADY=$(jq -r --arg s "$SKILL_NAME" '.skills_used // [] | map(select(. == $s)) | length' "$SIDECAR_FILE" 2>/dev/null || echo "0")
if [ "$ALREADY" != "0" ]; then
  exit 0
fi

NOW=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Append skill to skills_used and update timestamp
jq --arg s "$SKILL_NAME" --arg now "$NOW" \
  '.skills_used = ((.skills_used // []) + [$s]) | .updated_at = $now' \
  "$SIDECAR_FILE" > "$SIDECAR_FILE.tmp" && mv "$SIDECAR_FILE.tmp" "$SIDECAR_FILE"

exit 0
