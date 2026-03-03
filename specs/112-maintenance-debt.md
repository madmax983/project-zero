# 112: Maintenance Debt

## 1. Overview

Buildings are not eternal. Entropy slowly degrades all structures. Currently, pops repair buildings automatically when they have resources. This spec introduces the ability to **Defer Maintenance**, allowing players to prioritize resource saving over structural integrity.

However, ignoring maintenance has consequences. As a building's health drops, it accumulates **Malfunction Risk**. A malfunctioning building may stop production, lose resources, or catastrophic failure (explosion/fire).

This adds a strategic layer: "Do I spend my last 10 Metal repairing the Smelter, or do I risk it blowing up to build a new turret?"

## 2. Dependencies

- `src/layer1/structure.rs` — Structure component (HP).
- `src/layer1/building.rs` — Building entities.
- `src/layer1/notifications.rs` — Alerting the player.
- `src/layer1/fire.rs` — Consequence of malfunction.
- `src/layer1/utility_ai.rs` — Repair job logic.

## 3. RED Phase: Tests First

Write these tests in `src/layer1/structure_maintenance_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::GridPosition;
    use crate::layer1::structure::{Structure, DeferMaintenance, entropy_system, calculate_malfunction_risk, malfunction_system};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::utility_ai::{ActionType, evaluate_repair}; // Hypothetical, builder to adapt

    // Mock for testing
    #[derive(Resource)]
    struct MockRng {
        val: f32,
    }

    fn setup_world() -> World {
        let mut world = World::new();
        // Register necessary components/resources
        world
    }

    #[test]
    fn test_entropy_system_reduces_hp() {
        let mut world = setup_world();
        let building = world.spawn((
            Structure { current_hp: 100.0, max_hp: 100.0 },
            GridPosition { x: 0, y: 0 },
            Building { building_type: BuildingType::Housing },
        )).id();

        // Run entropy system
        entropy_system(&mut world);

        let structure = world.get::<Structure>(building).unwrap();
        assert!(structure.current_hp < 100.0, "Entropy should reduce HP");
    }

    #[test]
    fn test_defer_maintenance_component() {
        let mut world = setup_world();
        let building = world.spawn((
            Structure { current_hp: 50.0, max_hp: 100.0 },
            DeferMaintenance, // New component
            GridPosition { x: 0, y: 0 },
        )).id();

        // This component should exist
        assert!(world.get::<DeferMaintenance>(building).is_some());
    }

    #[test]
    fn test_repair_logic_ignores_deferred() {
        let mut world = setup_world();
        let building = world.spawn((
            Structure { current_hp: 50.0, max_hp: 100.0 },
            DeferMaintenance,
            GridPosition { x: 0, y: 0 },
            Building { building_type: BuildingType::Housing },
        )).id();

        // Evaluate repair job (conceptually)
        // This test ensures the utility AI query filters out DeferMaintenance
        // Implementation detail: Builder must ensure `evaluate_repair` or similar checks this.
        let mut query = world.query::<(&Structure, Without<DeferMaintenance>)>();
        let count = query.iter(&world).count();
        assert_eq!(count, 0, "Should not find building for repair if deferred");
    }

    #[test]
    fn test_malfunction_risk_calculation() {
        // 100% HP -> 0% Risk
        assert_eq!(calculate_malfunction_risk(100.0, 100.0), 0.0);

        // 50% HP -> 0% Risk (Threshold is 30%)
        assert_eq!(calculate_malfunction_risk(50.0, 100.0), 0.0);

        // 10% HP -> Positive Risk
        // (0.3 - 0.1) * 0.1 = 0.02 (2%)
        let risk = calculate_malfunction_risk(10.0, 100.0);
        assert!(risk > 0.0);
        assert!((risk - 0.02).abs() < f32::EPSILON);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### 1. Define `DeferMaintenance` Component

In `src/layer1/structure.rs`:

```rust
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct DeferMaintenance;
```

### 2. Implement `entropy_system`

In `src/layer1/structure.rs`:

```rust
pub fn entropy_system(world: &mut World) {
    let mut query = world.query::<(&mut Structure, Option<&Building>)>();

    for (mut structure, building) in query.iter_mut(world) {
        // Base decay rate
        let decay = 0.01; // 0.01 HP per tick.

        // Modifiers based on building type (Walls decay slower?)
        let modifier = if let Some(b) = building {
            match b.building_type {
                BuildingType::Wall | BuildingType::Gate => 0.1,
                _ => 1.0,
            }
        } else {
            1.0
        };

        structure.current_hp = (structure.current_hp - decay * modifier).max(0.0);
    }
}
```

### 3. Implement `malfunction_system` and helper

In `src/layer1/structure.rs`:

```rust
pub fn calculate_malfunction_risk(current: f32, max: f32) -> f32 {
    let percent = current / max;
    if percent < 0.3 {
        (0.3 - percent) * 0.1
    } else {
        0.0
    }
}

pub fn malfunction_system(world: &mut World) {
    let mut events = Vec::new();
    let mut query = world.query::<(Entity, &Structure, &GridPosition, Option<&Building>)>();

    for (entity, structure, pos, building) in query.iter(world) {
        let risk = calculate_malfunction_risk(structure.current_hp, structure.max_hp);

        if risk > 0.0 && rand::random::<f32>() < risk {
            events.push((entity, *pos));
        }
    }

    for (entity, pos) in events {
        // Trigger malfunction
        // e.g., Spawn Fire
        world.spawn((
            crate::layer1::fire::Fire { intensity: 1.0, lifetime: 10 },
            pos,
        ));

        // Notification
        if let Some(mut log) = world.get_resource_mut::<crate::shared::log::MessageLog>() {
            log.add(format!("Malfunction at ({}, {}) due to lack of maintenance!", pos.x, pos.y));
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Repair Logic Update:** The most critical part of this feature. Ensure `src/layer1/utility_ai.rs` (or wherever `evaluate_repair` lives) adds `Without<DeferMaintenance>` to its query.
- **Weather Integration:** `entropy_system` should query `WeatherState`. If `Storm`, decay is 5x.
- **Visuals:** Render "cracks" or "rust" on buildings with low HP in the TUI (maybe change color to red/brown).

## 6. Acceptance Criteria (Testable!)

- [ ] `DeferMaintenance` component exists.
- [ ] `entropy_system` slowly reduces HP of all buildings (approx 0.01/tick).
- [ ] `calculate_malfunction_risk` returns correct probabilities for low HP.
- [ ] `malfunction_system` uses the risk calculation to spawn bad events.
- [ ] Buildings with `DeferMaintenance` are excluded from auto-repair logic.
- [ ] `cargo test` passes.

## 7. Technical Guidance

- **Repair Job Generation:** You'll likely need to modify `src/layer1/utility_ai.rs` or `src/layer1/actions/repair.rs` to filter out `DeferMaintenance` entities.
- **Tuning:** `0.01` HP per tick means 10,000 ticks (several game days) to decay.
- **RNG:** Use `rand::thread_rng()` inside systems.

## 8. Questions

- *Builder: Does `DeferMaintenance` stop manual repairs?*
- *Architect:* Yes, `DeferMaintenance` blocks all repair jobs until toggled off.
