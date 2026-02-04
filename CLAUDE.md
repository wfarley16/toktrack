# toktrack

Ultra-fast AI CLI token usage tracker. Rust + simd-json + ratatui.

## Quick Start
```bash
cargo build --release
./target/release/toktrack
```

## Context
| File | Content |
|------|---------|
| `.claude/ai-context/architecture.md` | Layers, paths, traits, data flow |
| `.claude/ai-context/conventions.md` | Naming, TDD, error handling, commits |

## Workflow
```
/clarify → Plan Mode → /implement → /verify → /review → /wrap
```
각 단계 완료 후 즉시 다음 호출. 확인 묻지 말 것.

## Commands
```bash
make check    # fmt + clippy + test (pre-commit)
cargo test    # Run tests
cargo bench   # Benchmarks
```

## CI/CD
```
PR → CI (3 OS) → main → release-please → Release PR → 5 platform builds + npm
```

## Commit Rules
```
{type}({scope}): {description}
```
types: `feat|fix|refactor|docs|test|chore|perf`
scopes: `parser|tui|services|cache|cli`
