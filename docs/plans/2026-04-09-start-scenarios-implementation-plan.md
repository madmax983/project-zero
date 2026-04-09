# Start Scenarios Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a scenario-driven startup framework, add scenario specs and tracking artifacts, and implement the first plumbing slice without breaking the current default opening.

**Architecture:** Keep the shared landing layout and route startup through a new scenario model layered on top of `setup_world_with_config`. The framework introduces stable scenario identifiers, active-scenario tracking, and centralized scenario definitions now; contentful scenario mutations land in follow-up specs.

**Tech Stack:** Rust 2024, `bevy_ecs`, existing `setup.rs` startup path, markdown specs and planning docs

---

### Task 1: Write the scenario implementation specs and tracking docs

**Files:**
- Create: `docs/plans/2026-04-09-start-scenarios-implementation-plan.md`
- Create: `specs/882-start-scenario-framework.md`
- Create: `specs/883-start-scenario-selection.md`
- Create: `specs/884-ground-survival-start.md`
- Create: `specs/885-social-drama-start.md`
- Create: `specs/886-layer2-ready-start.md`
- Create: `specs/887-start-scenario-verification.md`
- Modify: `design/BACKLOG.md`
- Modify: `design/IN_PROGRESS.md`

**Step 1: Save this implementation plan**

Create the plan file and keep the scope aligned with the validated roadmap.

**Step 2: Create six atomic specs**

Write one spec per roadmap slice so each can be implemented in one focused session.

**Step 3: Update tracking docs**

Add `883` through `887` to `design/BACKLOG.md` and claim `882` in `design/IN_PROGRESS.md`.

**Step 4: Verify doc changes**

Run: `git diff --check`
Expected: no whitespace or patch errors

### Task 2: Add failing tests for the framework slice

**Files:**
- Modify: `src/setup.rs`

**Step 1: Write the failing tests**

Add tests that prove:
- `SetupConfig::default()` selects the classic start scenario
- `setup_world_with_config` records the selected scenario in world state
- scenario definitions exist for each built-in scenario identifier

**Step 2: Run the targeted tests to verify they fail**

Run: `cargo test test_setup_world_default_scenario_is_classic`
Run: `cargo test test_setup_world_records_selected_scenario`
Run: `cargo test test_built_in_start_scenarios_have_definitions`
Expected: FAIL because the scenario framework types and setup hooks do not exist yet

### Task 3: Implement the framework slice minimally

**Files:**
- Modify: `src/setup.rs`

**Step 1: Add startup scenario types**

Introduce `StartScenarioId`, `StartScenarioDefinition`, and an `ActiveStartScenario` resource.

**Step 2: Extend setup configuration**

Add `scenario` to `SetupConfig`, default it to `Classic`, and thread it through `setup_world_with_config`.

**Step 3: Insert active scenario state during setup**

Resolve the selected definition during setup and store it in world state without changing the current physical landing flow.

**Step 4: Run the targeted tests**

Run: `cargo test test_setup_world_default_scenario_is_classic`
Run: `cargo test test_setup_world_records_selected_scenario`
Run: `cargo test test_built_in_start_scenarios_have_definitions`
Expected: PASS

### Task 4: Guard the default opening

**Files:**
- Modify: `src/setup.rs`

**Step 1: Run the existing startup tests**

Run: `cargo test test_setup_world_spawns_pops`
Run: `cargo test test_setup_world_has_chronicle_event`
Run: `cargo test test_setup_world_generates_system`
Expected: PASS

**Step 2: Refactor lightly if needed**

Keep the old startup behavior intact for `Classic`. Do not implement menu selection or scenario-specific resource tuning in this task.

### Task 5: Finish the claimed slice cleanly

**Files:**
- Modify: `design/IN_PROGRESS.md`
- Modify: `design/COMPLETED.md`
- Modify: `docs/adr/*` if startup architecture needs a recorded decision

**Step 1: Verify the final diff**

Run: `git diff --check`
Expected: clean

**Step 2: Move `882` from in-progress to completed**

Only after the tests pass and the slice is actually implemented.

**Step 3: Commit**

```bash
git add docs/plans/2026-04-09-start-scenarios-implementation-plan.md specs/882-start-scenario-framework.md specs/883-start-scenario-selection.md specs/884-ground-survival-start.md specs/885-social-drama-start.md specs/886-layer2-ready-start.md specs/887-start-scenario-verification.md design/BACKLOG.md design/IN_PROGRESS.md design/COMPLETED.md src/setup.rs
git commit -m "feat(start): add scenario startup framework"
```
