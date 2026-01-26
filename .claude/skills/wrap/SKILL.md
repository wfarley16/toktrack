---
name: wrap
description: 세션 종료 - 문서 업데이트, 커밋, 다음 작업 정리
---

# Wrap

## Flow
```
Git Status → Doc Update Check → User Selection → Execute
```

## Execution

1. **Git Status**: 변경사항 확인
   ```bash
   git status --short
   git diff --stat HEAD~3
   ```

2. **Doc Update Check**:
   | 변경 | 대상 |
   |------|------|
   | 새 trait/타입 | ai-context/architecture.md |
   | 컨벤션 변경 | CLAUDE.md |
   | 새 모듈 | 해당 모듈 README |

3. **User Selection**: AskUserQuestion
   - 문서 업데이트 실행
   - 커밋 생성
   - 둘 다
   - 스킵

4. **Execute**: 선택된 작업 실행

## Commit Format
```
{type}: {summary}

Co-Authored-By: Claude <noreply@anthropic.com>
```

## Rules
- 미커밋 변경사항 확인
- 문서 동기화 필수
- 다음 세션 TODO 정리
