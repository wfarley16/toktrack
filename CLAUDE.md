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
| [architecture.md](.claude/ai-context/architecture.md) | Layers, paths, traits, data flow |
| [conventions.md](.claude/ai-context/conventions.md) | Naming, TDD, error handling, commits |

## Dev Workflow

```
/clarify → Plan Mode → /implement → /verify → /review → /wrap
```

See `.claude/skills/` for skill details.

## Commands

```bash
make check      # fmt + clippy + test (pre-commit)
make setup      # Configure git hooks
cargo test      # Run tests
cargo bench     # Benchmarks
```

## Commit Rules

```
{type}({scope}): {description}

Co-Authored-By: Claude <noreply@anthropic.com>
```

types: `feat|fix|refactor|docs|test|chore|perf`
scopes: `parser|tui|services|cache|cli`
