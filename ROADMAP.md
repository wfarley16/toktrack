# Roadmap

toktrack helps engineers **understand** their AI usage, **optimize** costs, and **justify** budgets. Today it delivers fast, unified tracking across multiple AI CLIs. Next, it will surface actionable insights — showing where money is spent, where it's wasted, and giving teams the data they need to defend AI tooling investment to leadership.

## Features Backlog

### P0 — Optimization Insights *(active focus)*

- **Model cost comparison** — "you spent $X on Opus; Sonnet would have cost $Y for these sessions"
- **Cache utilization metrics** — hit rate per session/day, cost impact of low cache usage
- **Cost-per-session visibility** — surface expensive sessions and explain why they cost more
- **Spending anomaly detection** — flag days/sessions with unusual spend

### P1 — Per-Project Attribution

- **Auto-detect project context** — link sessions to projects/repos via Claude Code project paths and git context
- **Cost rollup per project** — "project X cost $Y this month across Z sessions"
- **Expand session metadata** — add project/repo field to session sidecar
- **TUI view: per-project cost breakdown**

### P2 — Export & Reporting

- **CSV export** — `toktrack monthly --csv`
- **Exportable summaries** — shareable reports for stakeholders
- **HTML/PDF report generation** *(stretch)*

### P3 — Budget & Thresholds

- **Config file** — `~/.toktrack/config.toml`
- **Daily/monthly spending thresholds** with alerts
- **Budget tracking** — set a target, visualize actuals vs. budget

### Future / Exploring

- More CLI integrations (Cursor, Windsurf, Aider, VSCode Copilot)
- Team analytics (multi-user aggregation)
- Web dashboard / API
- Webhook integrations

## Tech Debt

### P0

- Fix `input_tokens` semantics / pricing formula ([CODE_REVIEW 1.6](CODE_REVIEW.md#16-medium-input_tokens-의미론과-memorymd-불일치))
- Fix `Some(0.0)` cost handling ([CODE_REVIEW 1.7](CODE_REVIEW.md#17-medium-some00-cost를-미계산으로-취급))
- Per-parser cache version checking ([CODE_REVIEW 1.4](CODE_REVIEW.md#14-medium-캐시-버전-mismatch-시-일부-parser가-구버전-캐시-사용))

### P1

- daily/weekly/monthly initial tab behavior ([CODE_REVIEW 1.3](CODE_REVIEW.md#13-high-toktrack-dailyweeklymonthly-tui-진입-시-overview-탭으로-시작))
- Home dir fallback → error instead of `.` ([CODE_REVIEW 1.8](CODE_REVIEW.md#18-low-home-directory-fallback이-pathbuffrom--파서-3곳))
- README sync ([CODE_REVIEW D1–D6](CODE_REVIEW.md#21-readme-계열-3개-파일-공통))
- Architecture & conventions doc sync ([CODE_REVIEW A1–A3, C1–C2](CODE_REVIEW.md#22-claudeai-contextarchitecturemd))

### P2

- Update overlay comment fix ([CODE_REVIEW 1.9](CODE_REVIEW.md#19-low-업데이트-오버레이-주석-구현-불일치))
- Unsafe SAFETY comment improvements ([CODE_REVIEW 1.10](CODE_REVIEW.md#110-low-unsafe-safety-코멘트-불충분))
