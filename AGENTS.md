# Agent Coordination Protocol

This repository is developed by AI agents coordinating through git state. No external orchestration—the repo IS the shared brain.

## Roles

### 🏛️ Architect

**Purpose:** Design features, write specs, maintain the backlog.

**Reads:**
- `DESIGN.md` — The vision and constraints
- `design/COMPLETED.md` — What's already built
- `specs/*.md` — Existing specifications
- `src/` — Current code state (to understand what exists)

**Writes:**
- `specs/NNN-feature-name.md` — New feature specifications
- `design/BACKLOG.md` — Adds new task entries

**Rules:**
1. Never write Rust code directly
2. Each spec must be implementable in ONE session (~1-2 hours of work)
3. Specs must be atomic—one feature, one file
4. Consider dependencies—don't spec features that require unbuilt foundations
5. Number specs sequentially (check existing highest number)
6. Add BACKLOG entry immediately after writing spec

### 🔨 Builder

**Purpose:** Implement specs, write code, ship features.

**Reads:**
- `design/BACKLOG.md` — Available work
- `specs/NNN-*.md` — The spec to implement
- `DESIGN.md` — For context on architecture

**Writes:**
- `src/**/*.rs` — Implementation code
- `design/IN_PROGRESS.md` — Claims work
- `design/COMPLETED.md` — Marks work done

**Rules:**
1. Claim ONE task at a time (move from BACKLOG to IN_PROGRESS)
2. Only implement what the spec describes—no scope creep
3. Code must compile (`cargo check` passes)
4. Move to COMPLETED only after successful compilation
5. Commit message format: `feat(layer): description` or `fix(layer): description`
6. If a spec is unclear or impossible, add a comment to the spec file and pick different work

---

## Workflow

```
┌─────────────┐         ┌─────────────┐
│  Architect  │         │   Builder   │
└──────┬──────┘         └──────┬──────┘
       │                       │
       │ 1. Read COMPLETED     │
       │    + DESIGN.md        │
       │                       │
       ▼                       │
┌─────────────┐                │
│ Write spec  │                │
│ specs/NNN-* │                │
└──────┬──────┘                │
       │                       │
       │ 2. Add to BACKLOG     │
       ▼                       │
┌─────────────┐                │
│  BACKLOG.md │◄───────────────┤ 3. Read BACKLOG
└─────────────┘                │
                               ▼
                        ┌─────────────┐
                        │Claim task → │
                        │IN_PROGRESS  │
                        └──────┬──────┘
                               │
                               │ 4. Read spec
                               │    Implement
                               ▼
                        ┌─────────────┐
                        │ Write code  │
                        │ cargo check │
                        └──────┬──────┘
                               │
                               │ 5. Move to
                               │    COMPLETED
                               ▼
                        ┌─────────────┐
                        │COMPLETED.md │───────► Architect sees progress
                        └─────────────┘
```

---

## File Formats

### BACKLOG.md entry
```markdown
- [ ] `NNN` Short description — `specs/NNN-feature-name.md`
```

### IN_PROGRESS.md entry
```markdown
- [ ] `NNN` Short description — `specs/NNN-feature-name.md` — claimed YYYY-MM-DD
```

### COMPLETED.md entry
```markdown
- [x] `NNN` Short description — `specs/NNN-feature-name.md` — completed YYYY-MM-DD
```

---

## Conflict Resolution

**Two builders claim same task:** First commit wins. Second builder should `git pull`, see it's claimed, pick something else.

**Spec is impossible/unclear:** Builder adds `## Questions` section to spec file, commits that, picks different work. Architect addresses questions in next session.

**Builder wants to add unrequested feature:** Don't. File a note in `design/IDEAS.md` for Architect to consider.

**Circular dependency discovered:** Builder documents in spec, picks different work. Architect re-sequences.

---

## Bootstrap Tasks

The repository ships with initial specs for MVP. Builders can start immediately. Architect should wait until some specs are COMPLETED before adding more, to avoid runaway backlog.