# toktrack

Ultra-fast AI CLI token usage tracker. Built with Rust + simd-json + ratatui.

## Features

- **Blazing Fast**: 5-10x faster than Node.js alternatives
- **Beautiful TUI**: 4 views (Overview, Models, Daily, Stats)
- **Data Preservation**: Automatic backup before 30-day deletion
- **Multi CLI Support**: Claude Code, OpenCode, Codex, Gemini (planned)

## Installation

**Recommended (No Rust required):**
```bash
npx toktrack
# or
bunx toktrack
```

**Other options:**
```bash
# Rust developers
cargo install toktrack

# Direct download
# → github.com/jaehojang/toktrack/releases
```

## Usage

```bash
# Launch TUI (default)
toktrack

# Daily usage report
toktrack daily
toktrack daily --json

# Statistics
toktrack stats

# Manual backup
toktrack backup
```

## Supported AI CLIs

| CLI | Status | Data Location |
|-----|--------|---------------|
| Claude Code | ✅ MVP | `~/.claude/projects/` |
| OpenCode | 🔜 v1.1 | `~/.local/share/opencode/` |
| Codex CLI | 🔜 v1.2 | `~/.codex/sessions/` |
| Gemini CLI | 🔜 v1.3 | `~/.gemini/tmp/*/chats/` |

## Performance

Benchmarked on 2,000+ files / 2.9GB data:

| Tool | Time |
|------|------|
| ccusage (Node.js) | ~20s |
| ccusage (cached) | ~7s |
| **toktrack** | **< 500ms** |

## Data Preservation

Claude Code and Gemini CLI delete session data after 30 days by default.

toktrack automatically backs up your data to `~/.toktrack/cache/` on first run.

To disable auto-deletion in Claude Code:
```json
// ~/.claude/settings.json
{
  "cleanupPeriodDays": 9999999999
}
```

## Configuration

```toml
# ~/.toktrack/config.toml

[cache]
enabled = true
backup_on_start = true

[tui]
theme = "green"  # green, teal, blue, pink, purple, orange
```

## Development

```bash
# Run tests (TDD)
cargo test

# Run with watch
cargo watch -x test

# Benchmark
cargo bench

# Build release
cargo build --release
```

## License

MIT
