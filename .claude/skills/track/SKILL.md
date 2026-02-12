---
name: track
description: View or edit session metadata (issue, tags, notes)
---

# Track

## Flow
```
Read Session ID → Load Sidecar → Display/Edit → Save
```

## Execution

1. **Read Session ID**
   ```bash
   cat /tmp/toktrack-session-id
   ```
   If file missing → error: "No active session. Start a Claude Code session first."

2. **Load Sidecar**
   ```bash
   cat ~/.toktrack/sessions/<session-id>.json
   ```
   If missing → create minimal sidecar with session_id + timestamps.

3. **Quick Update** (if args provided)
   - `/track ISE-123` → set `issue_id` to `ISE-123`, save, print confirmation
   - `/track tag bug-fix` → append `bug-fix` to `tags`, save, print confirmation
   - `/track note "debugging flaky test"` → set `notes`, save, print confirmation

4. **Interactive** (no args)
   - Display current metadata in table format:
     ```
     Session: <session-id>
     Issue:   ISE-123
     Tags:    bug-fix, urgent
     Skills:  clarify → implement → verify
     Notes:   Debugging flaky test
     Branch:  feature/ISE-123-fix-bug
     ```
   - Use AskUserQuestion to ask what to update:
     - Issue ID
     - Add tag
     - Set notes
     - Clear tags

5. **Save**
   Write updated JSON to `~/.toktrack/sessions/<session-id>.json`
   Update `updated_at` timestamp.

## Rules
- Always preserve existing fields when updating
- Never overwrite `skills_used` (managed by hooks)
- Keep `auto_detected` intact
- Use `jq` or direct JSON write for updates
