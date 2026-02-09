# toktrack Refactoring Requirements

작성일: 2026-02-09
검토 범위: `src/`, `tests/`, `README.md`, `README.ko.md`, `npm/README.md`, `.claude/ai-context/*`, `MEMORY.md`

---

## 1. Code Issues

### 1.3 [High] `toktrack daily|weekly|monthly` TUI 진입 시 Overview 탭으로 시작

**파일**: `src/cli/mod.rs:62-95`, `src/tui/app.rs:150-151`

```rust
// cli/mod.rs:62-66 (Daily)
crate::tui::run(TuiConfig {
    initial_view_mode: DailyViewMode::Daily,
    initial_tab: None,       // ← None → Tab::default() → Overview
})
// Weekly, Monthly도 동일하게 initial_tab: None
```

```rust
// app.rs:150-151
view_mode: ViewMode::Dashboard {
    tab: config.initial_tab.unwrap_or_default(),  // → Tab::Overview
},
```

`DailyViewMode`는 설정되지만 **초기 탭은 항상 Overview**. 탭 3개(Overview/Stats/Models) 중 "Daily" 탭은 존재하지 않음.

**영향**: README의 "Daily tab (daily view)" 설명과 불일치. 사용자가 `toktrack daily` 실행 시 Overview가 보임.
**수정 옵션**:
- (A) Daily 탭을 4번째 탭으로 추가
- (B) daily/weekly/monthly 커맨드의 initial_tab을 Overview로 유지하되 문서 정정

---

### 1.4 [Medium] 캐시 버전 mismatch 시 일부 parser가 구버전 캐시 사용

**파일**: `src/services/data_loader.rs:78-84`, `src/services/cache.rs:210-232`

`has_valid_cache()`는 **어느 한 parser라도** 버전이 맞으면 `true` 반환:

```rust
// data_loader.rs:79-84
self.registry.parsers().iter()
    .any(|p| cs.is_version_current(p.name()))  // ← any → 하나만 맞아도 warm path 진입
```

Warm path에서 parser별 분기(`data_loader.rs:102`)는 **파일 존재 여부만** 체크. 버전 불일치 parser의 캐시도 `load_past_summaries()`에서 그대로 반환(warning 포함하지만 데이터는 사용):

```rust
// cache.rs:210-217, 222-230
let warning = if cache.version != CACHE_VERSION {
    Some(CacheWarning::VersionMismatch(...))
} else { None };
// ↓ warning 있어도 summaries는 반환
let summaries: Vec<DailySummary> = cache.summaries.into_iter()
    .filter(|s| s.date < today)
    .map(|mut s| { s.models = normalize_model_keys(s.models); s })
    .collect();
(summaries, warning)
```

**영향**: Parser A 버전 일치 + Parser B 버전 불일치 → B는 구버전 집계 로직의 캐시 데이터를 계속 사용
**수정**: parser별로 `is_version_current()` 검사, mismatch parser는 cold parse 강제

---

### 1.6 [Medium] `input_tokens` 의미론과 MEMORY.md 불일치

**파일**: `src/services/pricing.rs:217-228`, `.claude/.../memory/MEMORY.md`

```rust
// pricing.rs:217-218
let non_cached_input = entry.input_tokens.saturating_sub(entry.cache_read_tokens);
```

MEMORY.md: "`input_tokens` in Claude API = pure non-cached input (cache tokens are separate fields)"

**불일치**: 만약 MEMORY.md가 맞다면 `input_tokens`는 이미 non-cached이므로 `saturating_sub`은 이중 차감.
만약 코드가 맞다면 MEMORY.md 설명이 틀림.

실제 Claude API 문서에 따르면 **`input_tokens`는 non-cached 순수 입력** (캐시 토큰은 별도 필드). 따라서:
- 코드의 `saturating_sub`이 잘못됨 — 이중 차감으로 input cost 과소 계산
- 테스트(`pricing.rs:386-407`)도 잘못된 공식에 맞춰 작성됨

**실질적 영향 제한**: Claude Code 엔트리는 `costUSD`가 있어 이 공식을 타지 않음. 다른 parser(Codex, OpenCode)의 `input_tokens` 의미론이 다를 수 있어 parser별 분기가 필요할 수 있음.
**수정**: Claude Code 원본 JSONL의 실제 값으로 검증 후 공식 확정. MEMORY.md 또는 코드 정정.

---

### 1.7 [Medium] `Some(0.0)` cost를 "미계산"으로 취급

**파일**: `src/services/data_loader.rs:260`

```rust
} else if entry.cost_usd.is_none() || entry.cost_usd == Some(0.0) {
    if let Some(p) = pricing {
        entry.cost_usd = Some(p.calculate_cost(&entry));
    }
}
```

**문제**: `Some(0.0)`은 무료 모델의 정당한 비용일 수 있음. copilot은 line 258에서 선처리되지만, 다른 무료 모델의 `Some(0.0)`은 재계산됨.
**수정**: `entry.cost_usd.is_none()`만 검사. `Some(0.0)`은 신뢰.

---

### 1.8 [Low] Home directory fallback이 `PathBuf::from(".")` — 파서 3곳

**파일**: `src/parsers/claude.rs:47`, `gemini.rs:52`, `opencode.rs:57`

```rust
let home = directories::BaseDirs::new()
    .map(|d| d.home_dir().to_path_buf())
    .unwrap_or_else(|| PathBuf::from("."));  // ← 현재 디렉토리에서 스캔
```

반면 `cache.rs:63`은 올바르게 에러 반환. 파서만 일관성 없이 `.` 폴백.
**수정**: 에러 반환 또는 빈 결과 반환 (조용한 오작동 방지)

---

### 1.9 [Low] 업데이트 오버레이 주석-구현 불일치

**파일**: `src/tui/app.rs:448, 461`

```rust
// 448: "q/Esc to quit"  ← 주석
// 461: KeyCode::Esc만 처리  ← 실제 구현 (q 없음)
```

**수정**: 주석을 `Esc to dismiss`로 정정

---

### 1.10 [Low] unsafe SAFETY 코멘트 불충분

**파일**: `src/parsers/gemini.rs:87-90`, `opencode.rs:93-96`

```rust
// SAFETY: simd_json requires mutable access for in-place parsing
let session: GeminiSession = unsafe {
    simd_json::from_str(&mut content)...
};
```

`unsafe`는 정당함(`simd_json::from_str`이 `pub unsafe fn`). 하지만 SAFETY 코멘트가 "왜 안전한가"가 아닌 "왜 필요한가"만 설명. 관례상 "buffer is exclusively owned, not shared across threads"와 같은 safety guarantee 명시 필요.

---

## 2. Documentation-Code Inconsistencies

### 2.1 README 계열 (3개 파일 공통)

| # | 문서 위치 | 문서 내용 | 코드 실제 | 영향 |
|---|-----------|-----------|-----------|------|
| D1 | `README.md:44`, `README.ko.md:40`, `npm/README.md:35` | "4 views (Overview, Models, Daily, Stats)" | 탭 3개: Overview/Stats/Models. Daily는 별도 탭 아님 (`tabs.rs:14`) | 사용자 혼란 |
| D2 | `README.md:118`, `README.ko.md:114` | "`1-4` 키로 탭 전환" | `1-3`만 지원 (`tabs.rs:54-61`) | 기능 오해 |
| D3 | `README.md:124`, `README.ko.md:120` | "`q` 종료" | `q`는 메인 화면에서 동작 안 함 (`app.rs` 테스트 1041: `test_q_key_does_nothing`). 종료는 `Ctrl+C` (`help.rs:148`) | 사용자 불편 |
| D4 | `README.md:102-105`, `README.ko.md:98-101` | "`toktrack daily` → Daily 탭" | Overview 탭으로 시작 (`cli/mod.rs:64: initial_tab: None`) | 기능 오해 |
| D5 | `README.md:133`, `README.ko.md:129`, `npm/README.md:47` | OpenCode: `~/.local/share/opencode/storage/` | 실제: `.../storage/message/` + `**/msg_*.json` (`opencode.rs:53,87`) | 경로 부정확 |
| D6 | `README.md:198`, `README.ko.md:194` | warm path = "24h mtime filter" | 어제 00:00 로컬시간 기준 (`data_loader.rs:20-28`) | 기술적 부정확 |

### 2.2 `.claude/ai-context/architecture.md`

| # | 문서 위치 | 문서 내용 | 코드 실제 |
|---|-----------|-----------|-----------|
| A1 | `architecture.md:59` | `parse_recent_files(24h)` | 어제 00:00 로컬시간 (`data_loader.rs:20`) |
| A2 | `architecture.md:88-91` | `pricing.json`이 `cache/` 하위 | `~/.toktrack/pricing.json` (루트, `pricing.rs:127`) |
| A3 | `architecture.md` TUI Widgets 테이블 | `update_popup.rs` 미기재 | `src/tui/widgets/update_popup.rs` 존재 |

### 2.3 `.claude/ai-context/conventions.md`

| # | 문서 위치 | 문서 내용 | 코드 실제 |
|---|-----------|-----------|-----------|
| C1 | `conventions.md:7` | 파일 예시 `claude_parser.rs` | 실제: `claude.rs` |
| C2 | `conventions.md:10` | 상수 예시 `DEFAULT_CACHE_DIR` | 해당 상수 없음 |
| C3 | `conventions.md:29` | "No `anyhow` in library code" | `cli/mod.rs:55`, `tui/app.rs:777,862`에서 `anyhow::Result` 사용 (boundary code이므로 borderline) |

### 2.4 MEMORY.md

| # | 내용 | 실제 |
|---|------|------|
| M1 | "`input_tokens` in Claude API = pure non-cached input" | 코드는 `input_tokens - cache_read_tokens`로 계산 (`pricing.rs:218`). 둘 중 하나가 틀림 (1.6 참조) |
| M2 | "`data_loader.rs:load()` checks for `VersionMismatch` warning and falls back to `load_cold_path()`" | 실제 가드는 `has_valid_cache()` (`data_loader.rs:66-72`). warm path 내부가 아닌 진입 전에 판단. 설명이 부정확 |

---

## 3. Refactoring Priority

### Phase 1: Data Correctness (High)
| ID | 항목 | 난이도 |
|----|------|--------|
| 1.7 | `Some(0.0)` 조건 제거 → `is_none()`만 검사 | Low |
| 1.6 | `input_tokens` 의미론 검증 + 공식/문서 정정 | Medium |
| 1.3 | daily/weekly/monthly 초기 탭 설계 확정 | Medium |

### Phase 2: Robustness (Medium)
| ID | 항목 | 난이도 |
|----|------|--------|
| 1.4 | parser별 버전 검사 강제 | Medium |
| 1.8 | 파서 home dir 폴백 → 에러 반환 | Low |

### Phase 3: Documentation Sync (Low effort, High impact)
| ID | 항목 | 파일 수 |
|----|------|---------|
| D1-D6 | README 3개 파일 정정 | 3 |
| A1-A3 | architecture.md 정정 | 1 |
| C1-C2 | conventions.md 예시 업데이트 | 1 |
| M1-M2 | MEMORY.md 정정 | 1 |
| 1.9 | app.rs 주석 정정 | 1 |
| 1.10 | unsafe SAFETY 코멘트 보강 | 2 |

---

## 4. Test Status

```
cargo test: 366 passed, 0 failed, 1 ignored
```

주의: 일부 테스트가 잘못된 로직에 맞춰 작성됨:
- `pricing.rs:386-407` (`test_calculate_cost_with_cache_tokens`) — 1.6의 공식이 틀리면 이 테스트도 수정 필요
- `data_loader.rs:476-483` (`test_apply_pricing_zero_cost_triggers_recalculation`) — 1.7과 연동

---

## 5. Out of Scope

- `.dev/specs/*`: DRAFT/PLAN 히스토리 문서. 현재 코드와 drift 크지만 archived 성격으로 정합 대상 아님
- `docs/benchmarks/*`: 성능 기록 문서. 코드와 직접 충돌 없음
- `CLAUDE.md`: 워크플로 문서. 런타임 동작과 직접 충돌 없음
