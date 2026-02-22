---
name: todo-validation
description: Validate todo files against actual code state
invocation: /validate-todos [--all | --resolved | --pending | --ids 001,005]
---

# Todo Validation Skill

## Purpose

Systematically validate todo files by checking acceptance criteria against actual source code.

## Pre-conditions

1. Running in worktree (not main directory)
2. `todos/` directory exists with numbered todo files

## Workflow

### Phase 1: Setup

1. Confirm execution in worktree
2. List todo files: `ls todos/*.md | grep -E '[0-9]{3}-'`
3. Filter by scope (--all, --resolved, --pending, --ids)

### Phase 2: Batch Processing

For each batch of 5 todos:

1. **Read todo file**
   - Extract YAML front matter
   - Parse acceptance criteria checkboxes

2. **Validate criteria**
   For each criterion:
   - Identify type (file exists, pattern, function, test)
   - Execute verification
   - Record pass/fail

3. **Determine status**
   - All criteria met → resolved
   - Any unmet → pending

4. **Update if changed**
   - YAML status
   - Checkboxes
   - Work log entry
   - Filename (if status changed)

5. **Record result**

### Phase 3: Reporting

After each batch:
```
Batch N/M (todos XXX-YYY)
--------------------------
✓ XXX - Title: unchanged (status, X/Y criteria)
→ XXX - Title: old → new (reason)
✗ XXX - Title: old → new (reason)

Continue? [y/n]
```

Final summary:
```
Todo Validation Summary
========================
Processed: N todos
Changes: M files updated

| ID | Title | Old | New | Changes |
|----|-------|-----|-----|---------|
| ... | ... | ... | ... | ... |

Files updated:
- todos/XXX-status-priority-title.md
```

## Validation Methods

| Type | Check |
|------|-------|
| `file:line` exists | `ls` + `Read` at offset |
| Function `name` exists | `Grep "fn name"` |
| Pattern present | `Grep "pattern"` |
| Pattern absent | `Grep` returns 0 matches |
| Test passes | `cargo test name` |

## File Update Rules

When updating a todo file:

1. **YAML front matter**: Update `status` field
2. **Checkboxes**: Mark `[x]` or `[ ]` based on validation
3. **Work log**: Add row with date, "Validated", results
4. **Filename**: Rename if status changed
   - `mv todos/001-pending-p1-title.md todos/001-resolved-p1-title.md`

## Examples

### Command Variations

```bash
# Validate all todos
/validate-todos --all

# Only resolved (check for regressions)
/validate-todos --resolved

# Only pending (check if now complete)
/validate-todos --pending

# Specific todos
/validate-todos --ids 001,005,012
```

### Validation Output Examples

```
Batch 1/6 (todos 001-005)
--------------------------
✓ 001 - Get program accounts: unchanged (resolved, 3/3 criteria)
✓ 002 - Error sanitization: unchanged (resolved, 2/2 criteria)
→ 003 - Missing validation: pending → resolved (all criteria now met)
✓ 004 - Type safety: unchanged (resolved, 1/1 criteria)
✗ 005 - Dead code annotations: resolved → pending (criterion 2 regressed)

Continue? [y]
```

## Post-Validation

After validation completes:

1. Show final summary
2. Commit changes (if any):
   ```bash
   git add todos/
   git commit -m "chore: update todo statuses after validation"
   ```
3. Ask user: merge to main, keep branch, or abandon?
