# 691 The Exile

## 1. Overview
**Layer:** 1 -> 3
**Fantasy:** Sometimes the only solution is to make someone else's problem.
**Mechanic:** Instead of execution or imprisonment, you can "Banish" a troublesome Pop. They leave the map. Years later, they may return as a Pirate Captain, a wealthy Merchant, or a beggar, with traits reflecting their exile.
**Emergence:** You banish a thief who stole food. Ten years later, a Pirate Dreadnought hails you. It's him. He wants his food back.
**Tension:** Mercy (banishment) vs. Finality (execution).

## 2. Dependencies
- Core `Pop` and `Action` ECS mechanics (Layer 1).
- Justice System / Unrest mechanics (for crime evaluation).
- `Chronicle` / Event system for logging the banishment and potential return.
- `NarrativeGenerator` to craft the return event.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::social::justice::{BanishmentState, Crime, ExiledPop};
    use crate::shared::simulation_time::SimulationTime;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(SimulationTime::default());
        app.add_systems(Update, (process_banishments, evaluate_exile_returns));
        app
    }

    #[test]
    fn test_banish_pop_removes_from_layer1() {
        let mut app = setup_app();

        let pop_entity = app.world_mut().spawn((
            Pop::default(),
            Crime { severity: 5 },
            BanishmentState::Pending,
        )).id();

        app.update();

        // Assert: The pop entity should no longer have the `Pop` component,
        // or should have been moved out of the active simulation pool and tagged as `ExiledPop`.
        assert!(app.world().get::<Pop>(pop_entity).is_none());
        assert!(app.world().get::<ExiledPop>(pop_entity).is_some());
    }

    #[test]
    fn test_exiled_pop_returns_after_years() {
        let mut app = setup_app();

        let pop_entity = app.world_mut().spawn((
            ExiledPop {
                exiled_at_tick: 0,
                base_crime_severity: 5,
            },
        )).id();

        // Fast forward simulation time by several years (ticks)
        let mut sim_time = app.world_mut().resource_mut::<SimulationTime>();
        sim_time.tick = 1_000_000; // Simulated time lapse

        app.update();

        // Assert: A return event or component flag should indicate the pop is trying to return
        let exiled_pop = app.world().get::<ExiledPop>(pop_entity).unwrap();
        assert!(exiled_pop.has_returned);
        assert!(exiled_pop.return_role.is_some()); // e.g., Pirate, Merchant
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED → GREEN

// Builder: Implement `BanishmentState`, `ExiledPop`, and `Crime` components.
// Builder: Implement `process_banishments` system to remove `Pop` component and insert `ExiledPop`.
// Builder: Implement `evaluate_exile_returns` system to check `SimulationTime` against `exiled_at_tick`.
```

## 5. REFACTOR Phase: Quality & Design
- **Integration with Chronicle:** When an exile returns, trigger a `NarrativeEvent` for the UI/Chronicle so the player is notified of their return.
- **Return Role Logic:** Expand the `return_role` calculation to be deterministic but varied (e.g., using a hash of the pop's entity ID and `exiled_at_tick`) to determine if they return as a Merchant, Pirate, or Beggar based on `base_crime_severity`.
- **Memory Optimization:** Instead of keeping entity IDs alive forever for exiles, consider serializing them into a `Res<ExileRegistry>` to save ECS entity slots if the colony scales massively.
- **Tuning:** Ensure the return time is randomized slightly or tied to Layer 2/3 ship arrivals.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures for the new logic.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [ ] A `Pop` marked for `BanishmentState::Pending` is successfully converted to an `ExiledPop` and loses their `Pop` component.
- [ ] An `ExiledPop` successfully triggers a return state after a sufficient simulation time delay.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- **Location:** Implement in `src/layer1/social/justice.rs` or a new `exile.rs` submodule.
- **Hooking Up Systems:** Register `process_banishments` and `evaluate_exile_returns` in `src/layer1/systems/social.rs`.
- **Gotchas:** Be careful when stripping the `Pop` component; ensure any attached UI trackers, workstation assignments, or housing allocations are also cleanly dropped to avoid ghost assignments.
- **Future-proofing:** The `return_role` string should eventually map to an Enum (`ExileReturnRole`) to trigger specific gameplay effects (e.g., spawning a hostile Layer 2 fleet if `Pirate`).

## 8. Questions
*Builder: add questions here if spec is unclear.*
