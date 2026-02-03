# toktrack

[![CI](https://github.com/mag123c/toktrack/actions/workflows/ci.yml/badge.svg)](https://github.com/mag123c/toktrack/actions/workflows/ci.yml)
[![npm](https://img.shields.io/npm/v/toktrack)](https://www.npmjs.com/package/toktrack)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

[English](README.md) | **한국어**

**모든 AI 코딩 CLI**의 토큰 사용량과 비용을 한 곳에서 — Claude Code, Codex CLI, Gemini CLI 통합 대시보드.

Rust + simd-json + ratatui 기반 초고속 성능.

![toktrack overview](assets/demo.gif)

> *"Claude Code에 얼마나 쓰고 있지?"* — 궁금했다면, toktrack이 1초 안에 답을 줍니다. 기존 도구로 2,000개 이상의 세션 파일(3 GB)을 스캔하면 40초 이상 걸렸습니다 — toktrack은 ~1초면 됩니다.

## 주요 기능

- **초고속 파싱** — simd-json + rayon 병렬 처리 (~3 GiB/s 처리량)
- **TUI 대시보드** — 4개 뷰 (Overview, Models, Daily, Stats), 일별/주별/월별 집계
- **CLI 명령어** — `daily`, `weekly`, `monthly`, `stats` (JSON 출력 지원)
- **멀티 CLI 지원** — Claude Code, Codex CLI, Gemini CLI 한 곳에서
- **데이터 보존** — CLI 데이터 삭제 후에도 비용 기록 유지

## 설치

### npx (권장)

Rust 툴체인 불필요. 플랫폼에 맞는 바이너리를 자동으로 다운로드합니다.

```bash
npx toktrack
# 또는
bunx toktrack
```

### 소스에서 설치

```bash
cargo install --git https://github.com/mag123c/toktrack
```

### 미리 빌드된 바이너리

[GitHub Releases](https://github.com/mag123c/toktrack/releases)에서 다운로드하세요.

| 플랫폼 | 아키텍처 |
|---------|----------|
| macOS | x64, ARM64 |
| Linux | x64, ARM64 |
| Windows | x64 |

## 빠른 시작

```bash
# TUI 대시보드 실행
npx toktrack

# 오늘의 비용을 JSON으로 확인
npx toktrack daily --json

# 월별 요약
npx toktrack monthly --json
```

## 사용법

### TUI 모드 (기본)

```bash
toktrack
```

### CLI 명령어

```bash
# 특정 탭으로 TUI 열기
toktrack daily     # Daily 탭 (일별 보기)
toktrack weekly    # Daily 탭 (주별 보기)
toktrack monthly   # Daily 탭 (월별 보기)
toktrack stats     # Stats 탭

# JSON 출력 (스크립팅용)
toktrack daily --json
toktrack weekly --json
toktrack monthly --json
toktrack stats --json
```

### 키보드 단축키

| 키 | 동작 |
|-----|--------|
| `1-4` | 탭 직접 전환 |
| `Tab` / `Shift+Tab` | 다음 / 이전 탭 |
| `j` / `k` 또는 `↑` / `↓` | 위 / 아래 스크롤 |
| `d` / `w` / `m` | 일별 / 주별 / 월별 보기 (Daily 탭) |
| `?` | 도움말 토글 |
| `q` | 종료 |

## 지원하는 AI CLI

| CLI | 상태 | 데이터 위치 |
|-----|--------|---------------|
| Claude Code | ✅ | `~/.claude/projects/` |
| Codex CLI | ✅ | `~/.codex/sessions/` |
| Gemini CLI | ✅ | `~/.gemini/tmp/*/chats/` |
| OpenCode | 🔜 | `~/.local/share/opencode/` |

## 성능

| 모드 | 시간 |
|------|------|
| 콜드 스타트 (캐시 없음) | **~1.0초** |
| 웜 스타트 (캐시 있음) | **~0.04초** |

> Apple Silicon 기준 (9,000+ 파일 / 3.4 GB).

## 데이터 보존

AI CLI들은 자체적으로 세션 데이터를 삭제하거나 순환합니다. toktrack은 일별 비용 요약을 독립적으로 캐시하므로, 원본 데이터가 사라진 후에도 사용 기록이 보존됩니다.

### CLI별 데이터 보존 정책

| CLI | 기본 보존 기간 | 정책 |
|-----|----------------|------|
| Claude Code | **30일** | `cleanupPeriodDays` (기본값: 30) |
| Gemini CLI | 무제한 | opt-in `sessionRetention` |
| Codex CLI | 무제한 | 용량 제한만 (`max_bytes`) |

### toktrack 캐시 구조

```
~/.toktrack/
├── cache/
│   ├── claude-code_daily.json   # 일별 비용 요약
│   ├── codex_daily.json
│   └── gemini_daily.json
└── pricing.json                 # LiteLLM 가격 정보 (1시간 TTL)
```

각 `*_daily.json`의 지난 날짜 데이터는 **불변**입니다 — 한번 집계된 날의 결과는 수정되지 않습니다. 현재 날짜만 매 실행마다 재계산됩니다. 따라서 Claude Code가 30일 후 세션 파일을 삭제하더라도, 캐시에 비용 기록이 그대로 남습니다.

### Claude Code 자동 삭제 비활성화

```json
// ~/.claude/settings.json
{
  "cleanupPeriodDays": 9999999999
}
```

### 캐시 초기화

```bash
rm -rf ~/.toktrack/cache/
```

다음 실행 시 사용 가능한 세션 데이터로부터 캐시를 재구축합니다.

## 동작 방식

![architecture](assets/architecture.png)

**콜드 경로** (첫 실행): 전체 glob 스캔 → 병렬 SIMD 파싱 → 캐시 구축 → 집계.

**웜 경로** (캐시 있음): 캐시된 요약 로드 → 최근 파일만 파싱 (24시간 mtime 필터) → 병합 → 집계.

> **Deep Dive:** [Node.js CLI를 Rust로 재작성 — 43초에서 1초로](https://mag1c.tistory.com/601) | [English](https://medium.com/@diehreo/i-rewrote-a-node-js-cli-in-rust-it-went-from-43s-to-1s-c13e38e7fe88)

## 개발

```bash
make check    # fmt + clippy + test (커밋 전 실행)
cargo test    # 테스트 실행
cargo bench   # 벤치마크 실행
```

## 로드맵

- [ ] **OpenCode 지원**

## 기여하기

이슈와 PR 환영합니다!

```bash
make check  # PR 전 실행
```

## 라이선스

MIT
