# Spec 1092: Obsessive Optimization

## 1. Overview
High-skill Engineers in the colony develop a "Tinker" urge. They will spontaneously attempt to dismantle and improve working machinery. If they succeed, the machine gains a permanent efficiency boost. If they fail, the machine is damaged or destroyed.

**Fantasy:** If it ain't broke, fix it until it is. The curse of the genius.

## 2. Dependencies
- Layer 1 Population (`Pop`, `SkillLevel`)
- Layer 1 Core Architecture (`Building`, `BuildingCondition`, `EfficiencyMultiplier`)
- Layer 1 Utility AI (`ActionType`, `ActionContext`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::architecture::building::{Building, BuildingCondition, EfficiencyMultiplier};
    use crate::layer1::pop::{Pop, SkillLevel};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, obsessive_optimization_system);
        app
    }

    #[test]
    fn test_tinker_success_boosts_efficiency() {
        let mut app = setup_app();

        // Target machine
        let machine = app.world_mut().spawn((
            Building::default(),
            BuildingCondition { health: 100.0, max_health: 100.0 },
            EfficiencyMultiplier { value: 1.0 },
        )).id();

        // Engineer pop with high skill
        let engineer = app.world_mut().spawn((
            Pop::default(),
            SkillLevel { engineering: 95 }, // Very high skill
            TinkeringTarget { target: machine },
            ForceTinkerOutcome { success: true }, // Test harness component
        )).id();

        app.update();

        let efficiency = app.world().get::<EfficiencyMultiplier>(machine).unwrap();
        assert!(efficiency.value > 1.0, "Successful tinker should boost efficiency");
    }

    #[test]
    fn test_tinker_failure_damages_machine() {
        let mut app = setup_app();

        // Target machine
        let machine = app.world_mut().spawn((
            Building::default(),
            BuildingCondition { health: 100.0, max_health: 100.0 },
            EfficiencyMultiplier { value: 1.0 },
        )).id();

        // Engineer pop with high skill but fails
        let engineer = app.world_mut().spawn((
            Pop::default(),
            SkillLevel { engineering: 80 },
            TinkeringTarget { target: machine },
            ForceTinkerOutcome { success: false }, // Test harness component
        )).id();

        app.update();

        let condition = app.world().get::<BuildingCondition>(machine).unwrap();
        assert!(condition.health < 100.0, "Failed tinker should damage the machine");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
- Create `src/layer1/tinkering.rs`.
- Define `TinkeringTarget { pub target: Entity }` and test harness `ForceTinkerOutcome { pub success: bool }`.
- Implement `obsessive_optimization_system`:
  - Query all `Pop`s with `SkillLevel` (where engineering > threshold, e.g., 80) and `TinkeringTarget`.
  - Check the target `Building`.
  - Resolve the tinkering outcome (use `ForceTinkerOutcome` if present, otherwise calculate probability based on `SkillLevel`).
  - If success: Increase `EfficiencyMultiplier`.
  - If failure: Decrease `BuildingCondition.health` (potentially triggering destruction if it reaches 0).
  - Remove the `TinkeringTarget` component from the pop.

## 5. REFACTOR Phase: Quality & Design
- **Integration with Utility AI:** Rather than a hardcoded component, tinkering should be an `ActionType` that high-skill engineers score highly when idle.
- **Limits:** There should be a cap on how much a building can be optimized to prevent infinite stacking.
- **Player Control:** Add a colony policy or individual toggle to "Forbid Unauthorized Maintenance" to prevent this behavior if the player wants safety over optimization.

## 6. Acceptance Criteria
- [ ] RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] High-skill pops occasionally alter building efficiency or health.
- [ ] Test coverage >= 85%.

## 7. Technical Guidance
- Make sure to use existing components for `BuildingCondition` and `EfficiencyMultiplier` (or equivalent if named differently in the current codebase).
- Ensure destroyed buildings trigger the usual destruction events.

## 8. Questions
*Builder: Add questions here if integration with the Utility AI for scheduling the tinker action is unclear.*

- *Architect: For the MVP, it's sufficient to implement this as an ECS system that reacts to the TinkeringTarget component. Integration with the Utility AI (making it an ActionType) can be deferred to the REFACTOR or a subsequent spec.*
