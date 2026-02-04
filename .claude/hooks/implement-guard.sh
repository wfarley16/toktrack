#!/bin/bash
# Edit/Write 호출 전 워크플로우 체크
# 코드 파일(*.rs, *.ts, *.js, *.py 등) 수정 시에만 체크
# 문서, 설정, 마케팅 자료 등은 직접 수정 허용
set -e
INPUT=$(cat)
SESSION_ID=$(echo "$INPUT" | jq -r '.session_id // "unknown"')
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // ""')

PLAN_EXITED_MARKER="/tmp/toktrack-plan-exited-$SESSION_ID"
IMPLEMENT_STARTED_MARKER="/tmp/toktrack-implement-started-$SESSION_ID"
CLARIFY_IN_PROGRESS_MARKER="/tmp/toktrack-clarify-in-progress-$SESSION_ID"

# 코드 파일 확장자 패턴
CODE_EXTENSIONS="rs|ts|tsx|js|jsx|py|go|java|kt|swift|c|cpp|h|hpp"

# 코드 파일인지 확인
is_code_file() {
  echo "$1" | grep -qE "\.($CODE_EXTENSIONS)$"
}

# 코드 파일이 아니면 통과 (문서, 설정 등)
if ! is_code_file "$FILE_PATH"; then
  exit 0
fi

# clarify 진행 중인데 Plan Mode 없이 코드 수정 시도 시 경고 (차단 아님)
if [ -f "$CLARIFY_IN_PROGRESS_MARKER" ]; then
  echo "⚠️ /clarify 후 Plan Mode 진행 권장 (코드 파일 수정)" >&2
  exit 0
fi

# Plan Mode 종료됐는데 implement 스킬 미실행 상태면 경고 (차단 아님)
if [ -f "$PLAN_EXITED_MARKER" ] && [ ! -f "$IMPLEMENT_STARTED_MARKER" ]; then
  echo "⚠️ 코드 구현은 /implement 스킬 사용 권장 (TDD 워크플로우)" >&2
  exit 0
fi

exit 0
