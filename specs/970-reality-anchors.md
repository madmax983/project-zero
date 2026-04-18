# Spec 970: Reality Anchors

## 1. Overview
As the colony researches and constructs highly advanced experimental technologies (like FTL hyper-relays or void energy taps), they begin to tear at the fabric of local physics. This introduces a new metric, `RealityStability`. High-tier experimental buildings consume `RealityStability`. If stability drops too low, the local grid suffers from Reality Anomalies (e.g., Pops randomly teleporting short distances). To counter this, the colony must build `RealityAnchor` structures, which consume massive amounts of `Energy` to generate `RealityStability`.

**Fantasy:** Science has gone too far. Physics is starting to peel at the edges.

## 2. Dependencies
- Layer 1 Core Architecture (`Building`, `GridPosition`)
- Layer 1 Population (`Pop`, `GridPosition`)
- Layer 1 Simulation (`ColonyResources`, specifically `Energy`)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::architecture::building::{Building, BuildingType};
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;

    fn setup_app() -> App {
        let mut app = App::new();
        app.insert_resource(ColonyResources::default());
        app.insert_resource(RealityStability { value: 100.0 });
        app.add_systems(Update, (
            drain_reality_stability_system,
            restore_reality_stability_system,
            reality_anomaly_teleport_system,
        ));
        app
    }

    #[test]
    fn test_experimental_building_drains_stability() {
        let mut app = setup_app();

        // Spawn an experimental building that drains stability
        app.world_mut().spawn((
            Building {
                b_type: BuildingType::HyperRelay, // A new type, or a generic tag
                ..default()
            },
            RealityDrain { amount: 10.0 },
        ));

        app.update();

        let stability = app.world().resource::<RealityStability>();
        assert_eq!(stability.value, 90.0, "Experimental building should drain reality stability");
    }

    #[test]
    fn test_reality_anchor_restores_stability_using_energy() {
        let mut app = setup_app();

        // Lower starting stability
        app.world_mut().resource_mut::<RealityStability>().value = 50.0;

        // Provide energy
        app.world_mut().resource_mut::<ColonyResources>().energy = 100;

        // Spawn a Reality Anchor
        app.world_mut().spawn((
            Building {
                b_type: BuildingType::RealityAnchor,
                ..default()
            },
            RealityAnchorData { restore_amount: 15.0, energy_cost: 20 },
        ));

        app.update();

        let stability = app.world().resource::<RealityStability>();
        assert_eq!(stability.value, 65.0, "Anchor should restore stability");

        let resources = app.world().resource::<ColonyResources>();
        assert_eq!(resources.energy, 80, "Anchor should consume energy");
    }

    #[test]
    fn test_reality_anchor_fails_without_energy() {
        let mut app = setup_app();

        app.world_mut().resource_mut::<RealityStability>().value = 50.0;
        app.world_mut().resource_mut::<ColonyResources>().energy = 0; // No energy!

        app.world_mut().spawn((
            Building {
                b_type: BuildingType::RealityAnchor,
                ..default()
            },
            RealityAnchorData { restore_amount: 15.0, energy_cost: 20 },
        ));

        app.update();

        let stability = app.world().resource::<RealityStability>();
        assert_eq!(stability.value, 50.0, "Anchor should not restore stability without energy");
    }

    #[test]
    fn test_low_stability_causes_teleport_anomalies() {
        let mut app = setup_app();

        // Critically low stability
        app.world_mut().resource_mut::<RealityStability>().value = -10.0;

        let original_pos = GridPosition { x: 10, y: 10, z: 0 };
        let pop_entity = app.world_mut().spawn((
            Pop::default(),
            original_pos.clone(),
        )).id();

        app.update();

        let new_pos = app.world().get::<GridPosition>(pop_entity).unwrap();
        assert_ne!(*new_pos, original_pos, "Pop should randomly teleport when reality stability is critical");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

- Create `src/layer1/reality.rs` (or `src/layer1/physics/reality.rs`).
- Define `RealityStability` Resource (default 100.0).
- Define `RealityDrain` Component (`amount: f32`).
- Define `RealityAnchorData` Component (`restore_amount: f32`, `energy_cost: u32`).
- Add `BuildingType::HyperRelay` and `BuildingType::RealityAnchor` to `BuildingType`.
- Implement `drain_reality_stability_system`:
  - Query all `RealityDrain`.
  - Subtract their total amount from `RealityStability`.
- Implement `restore_reality_stability_system`:
  - Query all `RealityAnchorData`.
  - For each, if `ColonyResources::energy >= energy_cost`, deduct energy and add `restore_amount` to `RealityStability`.
- Implement `reality_anomaly_teleport_system`:
  - If `RealityStability` < 0.0, query some `Pop` with `GridPosition`.
  - Randomly change their `GridPosition` by a small offset (e.g., +/- 2 tiles).

## 5. REFACTOR Phase: Quality & Design
- **Clamping:** Ensure `RealityStability` is clamped between a sensible range (e.g., -100.0 to 100.0) so anomalies don't spiral infinitely out of control.
- **Probability:** The teleportation should probably be probabilistic based on how negative the stability is, rather than affecting every pop every tick.
- **System Ordering:** Ensure drain runs before restore, and anomalies run after both.

## 6. Acceptance Criteria
- [ ] Tests compile and pass in RED phase.
- [ ] `RealityStability` accurately reflects the balance of Drains and Anchors.
- [ ] Anchors respect Energy costs and shut down if power is insufficient.
- [ ] Pops teleport randomly when Reality is compromised.
- [ ] Test coverage ≥85%.
- [ ] Code passes `clippy -- -D warnings` and `cargo fmt`.

## 7. Technical Guidance
- Add the new module to `src/layer1/mod.rs` (or `src/layer1/physics/mod.rs`).
- Register the systems in `src/simulation.rs` in the appropriate schedule (likely the `SimulationSchedule::Update` or a physics step).
- Ensure `RealityStability` is initialized in `src/setup.rs` to fix any headless/fresh-world test panics.
- Use `rand::Rng` for the random teleport offsets.

## 8. Questions
*Builder: add questions here if spec is unclear.*

- Spec requires `ColonyResources::energy`, but the `ColonyResources` struct does not have an `energy` field (energy is handled via the grid and `PowerConsumer` components, or perhaps `fuel` or `credits`). Adding `energy` directly to `ColonyResources` would duplicate or contradict the existing grid-based energy system architecture. Please clarify how Reality Anchors should consume energy (e.g., via a `PowerConsumer` component with a `demand`, or by consuming `ColonyResources::fuel` or `batteries`).
*Architect:* Reality Anchors should consume `ColonyResources::energy` directly if available in the model. If energy is modeled via `PowerConsumer`, add a `PowerConsumer` component to the Reality Anchor and only restore stability when the building is powered.
