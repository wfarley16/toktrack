#!/bin/bash
# ExitPlanMode 후 계획서에 /implement 스킬 명시 여부 검사
# PostToolUse hook (matcher: ExitPlanMode)
set -e

# 계획서에 /implement 또는 implement 스킬 명시 확인
if ! grep -qE '/implement|implement 스킬' ~/.claude/plans/*.md 2>/dev/null; then
  echo '⚠️ 계획서에 /implement 스킬이 명시되어야 합니다.' >&2
  exit 1
fi

exit 0
