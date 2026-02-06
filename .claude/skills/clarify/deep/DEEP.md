# Deep Path — Interview-Driven Planning

Full process for HIGH complexity or escalated from Shallow path.

---

## Interview Mode (Default)

### Step 1: Initialize

#### 1.1 Classify Intent

| Intent Type | Keywords | Strategy |
|-------------|----------|----------|
| **Refactoring** | "refactoring", "cleanup", "improve" | Safety first, regression prevention |
| **New Feature** | "add", "new", "implement" | Pattern exploration, integration points |
| **Bug Fix** | "bug", "error", "broken", "fix" | Reproduce → Root cause → Fix |
| **Architecture** | "design", "structure", "architecture" | Trade-off analysis |
| **Research** | "investigate", "analyze", "understand" | Investigation only, NO implementation |
| **Migration** | "migration", "upgrade", "transition" | Phased approach, rollback plan |
| **Performance** | "performance", "optimize", "slow" | Measure first, profile → optimize |

#### 1.2 Launch Parallel Exploration

Run 3 agents **in parallel within a single message**:

```
Task(subagent_type="Explore",
     prompt="Find: existing patterns for [feature type]. Report as file:line format.")

Task(subagent_type="Explore",
     prompt="Find: project structure, build/test/lint commands")

Task(subagent_type="Explore",
     prompt="Find internal documentation: ADRs, conventions, constraints, READMEs")
```

#### 1.3 Create Draft File

```
Write(".dev/specs/{name}/DRAFT.md", initial_draft)
```

Follow the structure in `templates/DRAFT_TEMPLATE.md`.

### Step 1.5: Present Exploration Summary

After parallel exploration completes, present summary to user before starting interview:

```
"Codebase exploration results:
 - Structure: [main directory structure]
 - Related patterns: [2-3 existing patterns found]
 - Internal docs: [related ADRs/conventions]
 - Project commands: lint/test/build

Please confirm this context is correct before proceeding."
```

### Step 2: Gather Requirements

#### ASK (Only the user knows)
- **Boundaries**: What must NOT be done
- **Trade-offs**: Multiple valid options
- **Success Criteria**: Completion conditions

#### DISCOVER (Agent explores)
- File locations, existing patterns, integration points, project commands

#### PROPOSE (Suggest based on research)
- Propose based on exploration results → User only approves/modifies

> **Key**: Minimize questions, maximize research-based proposals

### Step 3: Update Draft Continuously

- User answer → Update **User Decisions** table
- Exploration results → Update **Agent Findings**
- Resolved items → Remove from **Open Questions**
- Direction agreed → Update **Direction**

### Step 4: Plan Transition Check

Conditions:
- [ ] All Critical Open Questions resolved
- [ ] Key decisions recorded in User Decisions
- [ ] Success Criteria agreed
- [ ] User explicitly requests plan ("make the plan", "create plan", etc.)

**Do not generate a plan unless the user requests it.**

---

## Plan Generation Mode (On explicit request)

### Step 1: Validate Draft Completeness

Verify DRAFT contains:
- [ ] What & Why complete
- [ ] Boundaries specified
- [ ] Success Criteria defined
- [ ] Critical Open Questions empty
- [ ] Patterns and Commands in Agent Findings

If incomplete → Return to Interview Mode.

### Step 2: Run Parallel Analysis Agents

```
Task(subagent_type="general-purpose",
     prompt="Gap analysis: missing requirements, AI pitfalls, must-NOT-do items.
             Goal: [DRAFT What & Why]
             Current Understanding: [DRAFT summary]")

Task(subagent_type="general-purpose",
     prompt="Tradeoff analysis: risk per change area, simpler alternatives, dangerous changes.
             Proposed Approach: [DRAFT Direction]
             Boundaries: [DRAFT Boundaries]")
```

External research (only for migration, new libraries, or unfamiliar tech):
```
Task(subagent_type="general-purpose",
     prompt="Research official docs for [library/framework]: [specific question]")
```

**Gap analysis results**: Add to Must NOT Do
**Tradeoff analysis results**: Assign risk tags (LOW/MEDIUM/HIGH), HIGH requires user approval

### Step 3: Decision Summary Checkpoint

Present all decisions (user decisions + agent auto-decisions) before plan generation:

```
AskUserQuestion(
  question: "Please review the following decisions. Any items need modification?",
  options: [
    { label: "Confirmed", description: "All decisions are correct" },
    { label: "Needs changes", description: "I want to modify some items" }
  ]
)
```

### Step 4: Create Plan File

```
Write(".dev/specs/{name}/PLAN.md", plan_content)
```

Follow the structure in `templates/PLAN_TEMPLATE.md`.

#### DRAFT → PLAN Mapping

| DRAFT Section | PLAN Section |
|---------------|--------------|
| What & Why | Context > Original Request |
| User Decisions | Context > Interview Summary |
| Agent Findings | Context > Research Findings |
| Deliverables | Work Objectives > Concrete Deliverables |
| Boundaries | Work Objectives > Must NOT Do |
| Success Criteria | Work Objectives > Definition of Done |
| Agent Findings > Patterns | TODOs > References |
| Agent Findings > Commands | TODO Final > Verification commands |
| Direction > Work Breakdown | TODOs + Dependency Graph |

### Step 5: Reviewer

```
Task(subagent_type="feature-dev:code-reviewer",
     prompt="Review this plan: .dev/specs/{name}/PLAN.md")
```

### Step 6: Handle Reviewer Response

**REJECT (Cosmetic)** — Format, clarity, missing fields:
→ Auto-fix → Re-review → Repeat until OKAY

**REJECT (Semantic)** — Requirements, scope, logic changes:
→ Present to user → Modify per user choice → Re-review

Semantic criteria — The following changes are Semantic:
- Work Objectives (scope, deliverables, definition of done)
- TODO steps or acceptance criteria
- Risk level or rollback strategy
- Must NOT Do items

Everything else (wording, format, field completeness) → Cosmetic.

**OKAY**:
1. ~~Delete DRAFT~~ → **Preserve DRAFT** (for compounding)
2. **Append summary to DECISIONS.md**:
   ```markdown
   ## {date}: {feature-name}
   - **Decision**: {1-3 key items from User Decisions}
   - **Reason**: {Key point from Why}
   - **Reference**: .dev/specs/{name}/PLAN.md
   ```
3. Inform user that plan is ready
4. Call `EnterPlanMode()`

---

## Risk Tagging

| Risk | Meaning | Requirements |
|------|---------|--------------|
| LOW | Reversible, isolated | Standard verification |
| MEDIUM | Multiple files, API changes | Verify block + reviewer |
| HIGH | DB schema, auth, breaking API | Verify + rollback + human approval |

---

## File Locations

| Type | Path | When |
|------|------|------|
| Draft | `.dev/specs/{name}/DRAFT.md` | During interview |
| Plan | `.dev/specs/{name}/PLAN.md` | After plan generation |
