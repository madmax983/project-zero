# 882: Start Scenario Framework

**Layer:** Shared startup / Layer 1
**Status:** Draft
**Complexity:** Medium

---

## 1. Overview

**Fantasy:** The opening of the game should be selected, not assumed.

**Mechanic:**
- Introduce stable built-in start scenario identifiers.
- Extend `SetupConfig` so startup can request a specific scenario.
- Insert an `ActiveStartScenario` resource into the world during setup.
- Define startup scenario metadata in one place without changing the shared landing layout yet.

**Why:** The codebase currently has one hardcoded start path. Before we can add curated starts, startup needs a framework that selects and records a scenario without forking setup into three copy-pasted monstrosities.

---

## 2. Dependencies

- `026` Main Menu
- `078` The Old Guard
- `094` System View Architecture
- `146` Command Center & System Visibility

---

## 3. RED Phase: Tests First

Add these tests to `src/setup.rs` before implementation:

```rust
#[test]
fn test_setup_world_default_scenario_is_classic() {
    let config = SetupConfig::default();
    assert_eq!(config.scenario, StartScenarioId::Classic);
}

#[test]
fn test_setup_world_records_selected_scenario() {
    let world = setup_world_with_config(SetupConfig {
        headless: true,
        scenario: StartScenarioId::GroundSurvival,
    });

    let active = world.resource::<ActiveStartScenario>();
    assert_eq!(active.id, StartScenarioId::GroundSurvival);
}

#[test]
fn test_built_in_start_scenarios_have_definitions() {
    for id in StartScenarioId::all() {
        let definition = start_scenario_definition(id);
        assert_eq!(definition.id, id);
    }
}
```

Run the new tests and watch them fail because the scenario framework does not exist yet.

---

## 4. GREEN Phase: Minimal Implementation

### Add startup scenario types

Create framework types in `src/setup.rs`:

- `StartScenarioId`
- `StartScenarioDefinition`
- `ActiveStartScenario`

The first built-in IDs should be:

- `Classic`
- `GroundSurvival`
- `SocialDrama`
- `Layer2Ready`

### Extend setup configuration

Add `scenario: StartScenarioId` to `SetupConfig` and default it to `Classic`.

### Record the selected scenario during setup

Resolve the chosen definition at the start of `setup_world_with_config` and insert an `ActiveStartScenario` resource into the world. Do not change the current landing shell, pop positions, or starter building footprint in this slice.

---

## 5. REFACTOR Phase: Quality & Design

- Keep the scenario definition lookup centralized.
- Avoid scattering `match StartScenarioId` branches throughout setup.
- Make the types stable enough for later menu selection and save compatibility.

This slice is plumbing, not content. It should be boring on purpose.

---

## 6. Acceptance Criteria

- [ ] `SetupConfig` includes a start scenario field.
- [ ] Default setup selects `Classic`.
- [ ] Built-in start scenarios have centralized definitions.
- [ ] `setup_world_with_config` records the selected scenario in world state.
- [ ] Existing default startup tests still pass.

---

## 7. Technical Guidance

- Start with `src/setup.rs`; do not introduce a separate module unless the file genuinely becomes unmanageable.
- Prefer `const fn all()` or a fixed slice for enumerating built-in scenarios.
- Keep the selected scenario in a resource so later UI and chronicle systems can read it.

---

## 8. Questions

- *Builder: Should the framework apply any scenario mutations yet?*
  *Architect:* No. This slice only selects and records the scenario.
