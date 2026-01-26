# AI Context

Knowledge documents for toktrack project.

## Structure
```
ai-context/
├── architecture.md   # Layers, traits, data flow
├── conventions.md    # TDD, naming, errors, commits
└── CLAUDE.md         # This file
```

## Loading Rules
| Task | Load |
|------|------|
| All tasks | architecture.md |
| Writing code | + conventions.md |

## Core Principles
1. **TDD**: RED → GREEN → REFACTOR
2. **Performance**: simd-json + rayon
3. **Extensibility**: trait CLIParser

## Estimated Tokens
| File | Tokens |
|------|--------|
| architecture.md | ~500 |
| conventions.md | ~400 |
| **Total** | **~900** |

---
Last updated: 2026-01-26
