#!/bin/bash
# Plan Mode 완료 후 /implement 스킬 사용 안내 (강제 아님)
# 코드 구현 작업일 때만 TDD 워크플로우 권장
set -e
INPUT=$(cat)
SESSION_ID=$(echo "$INPUT" | jq -r '.session_id // "unknown"')
MARKER="/tmp/toktrack-plan-exited-$SESSION_ID"
IMPLEMENT_MARKER="/tmp/toktrack-implement-started-$SESSION_ID"

# Plan 완료 마커가 없거나, 이미 implement 시작했으면 종료
[ ! -f "$MARKER" ] && exit 0
[ -f "$IMPLEMENT_MARKER" ] && exit 0

# 안내만 제공 (강제 아님)
cat << 'EOF'
{
  "hookSpecificOutput": {
    "hookEventName": "UserPromptSubmit",
    "additionalContext": "## Plan Mode 완료\n\n코드 구현 작업이라면 `/implement` 스킬로 TDD 방식 권장.\n문서/마케팅 작업은 직접 진행 가능."
  }
}
EOF
