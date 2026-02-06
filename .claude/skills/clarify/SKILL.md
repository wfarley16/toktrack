---
name: clarify
description: |
  Adaptive requirements clarification with auto-depth routing.
  Shallow (Q&A) for simple tasks, Deep (exploration + DRAFT + PLAN) for complex ones.
  Escalates automatically when ambiguity persists.
required_context:
  - .claude/ai-context/architecture.md
  - .claude/ai-context/conventions.md
allowed-tools:
  - Read
  - Grep
  - Glob
  - Task
  - Write
  - AskUserQuestion
---

# /clarify — Adaptive Requirements Clarification

## Chain (MUST)
| Previous | Current | Next |
|----------|---------|------|
| Session start | /clarify | EnterPlanMode() → /implement |

## Core Model

```
/clarify → Measure complexity → Clear enough?
                                ├─ Yes → EnterPlanMode()
                                └─ No  → Deeper clarify (explore, analyze, DRAFT...)
                                         → Re-measure → Repeat
```

Exit condition: **"Is this enough info to implement?"**

---

## Step 0: Route — Complexity Assessment

Measure complexity internally upon receiving request (do not expose to user).

### Complexity Signals

| Signal | LOW | HIGH |
|--------|-----|------|
| Request length | Short and specific | Long or ambiguous |
| Keywords | "add", "fix", "change" | "design", "migration", "from scratch" |
| Uncertainty | None | "not sure", "how should I" |
| Impact scope | Single file/module | Cross-cutting, multiple services |
| Risk | Low (UI, text) | High (DB, auth, breaking API) |
| Existing patterns | Clearly exist | None or unfamiliar stack |

- **LOW** → Shallow Path (completed within this file)
- **HIGH** → Deep Path (see `deep/DEEP.md`)
- **Ambiguous** → Start Shallow, monitor escalation conditions

---

## Shallow Path (Low Complexity)

Remove ambiguity via quick Q&A and enter Plan Mode immediately.

### Execution

1. **Record**: Log original request + identify ambiguous parts
2. **Question**: `AskUserQuestion` (specific options, 2-3 rounds)
3. **Escalation Check**: Check escalation conditions (see below)
4. **Summary**: Before/After comparison (Goal, Scope, Constraints, Success Criteria)
5. **Auto Plan**: Call `EnterPlanMode()` immediately

### Rules
- No assumptions → Ask questions
- Clarify to TDD-ready level
- Target resolution within 3 rounds

### Escalation → Deep Path

Switch to Deep Path if any of these are detected:

- Scope still undefined after 3 rounds of questions
- New uncertainties keep emerging from user answers
- Impact scope expanded beyond initial estimate (single → multi-module)
- Risk indicators found (DB schema, auth, breaking changes)
- User doesn't know the approach itself ("I don't know how to do this")

On switch: Inform "Scope is more complex than expected. Exploring the codebase first." then follow the process in `deep/DEEP.md`.

---

## Deep Path

**When complexity is HIGH or escalated from Shallow.**

See `deep/DEEP.md` for the full process.

Summary:
1. Classify intent (7 types) → Determine strategy
2. 3 parallel exploration agents → Understand codebase
3. Generate DRAFT → Interview → Continuously update
4. On user explicit request → Analysis agents → Generate PLAN → Reviewer loop
5. After approval → `EnterPlanMode()`

---

## DECISIONS.md Recording (MUST)

Record decisions in `.dev/DECISIONS.md` before finalizing plan:

| Situation | Required Record |
|-----------|----------------|
| New feature design | Decision background, alternatives, reasoning |
| Architecture choice | Considered options, selection rationale |
| Trade-offs | What was sacrificed and what was gained |

```markdown
## YYYY-MM-DD: {feature-name}
- **Decision**: What was decided
- **Reason**: Why this choice was made
- **Alternatives**: Options considered but not chosen
- **Reference**: .dev/specs/{feature-name}/PLAN.md (if exists)
```

---

## Plan File Requirements

Plan files must include:
- Specify implementation via `/implement` skill
- Verification method (test execution)
- Confirmation that `.dev/DECISIONS.md` recording is complete

**Important**: Plans that do not use `/implement` will not be approved.

---

## NEXT STEP (Auto-execute)

On plan approval, call `/implement` **immediately**. Do not ask "Should I implement?".
