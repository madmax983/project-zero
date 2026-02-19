# 168: The Mother Lode

## Overview

Introduces the **Mother Lode**, a rare, procedurally generated ore node that provides infinite resources but becomes increasingly dangerous to mine.

As players mine the Mother Lode:
1.  **Infinite Yield**: It never depletes.
2.  **Rising Heat**: It emits increasing amounts of heat into the `TemperatureGrid` (`140`).
3.  **Rising Risk**: The `HazardLevel` increases, drastically raising the chance of accidents (`155`).

This creates a "Greed vs. Safety" tension: how deep can you dig before the mine collapses or cooks your workers?

## Dependencies

- `018` — Mining and Resources (for `mine_rock` and `ResourceItem`)
- `140` — Thermal Management (for `TemperatureGrid`)
- `155` — Advanced Workplace Hazards (for `calculate_risk`)

## RED Phase: Tests First

Write these tests in `src/layer1/mother_lode_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::mother_lode::{MotherLode, mining_mother_lode_system};
    use crate::layer1::resources::{ResourceType, ColonyResources};
    use crate::layer1::temperature::{TemperatureGrid, update_temperature_system};
    use crate::layer1::hazards::calculate_risk;
    use crate::layer1::skills::{Skills, SkillType};
    use crate::layer1::structure::Structure;
    use crate::layer1::map::GridPosition;
    use crate::layer1::building::{Building, BuildingType};

    #[test]
    fn test_mother_lode_initialization() {
        let lode = MotherLode {
            resource_type: ResourceType::Metal,
            current_hazard: 1.0,
            heat_output: 10.0,
        };
        assert_eq!(lode.resource_type, ResourceType::Metal);
        assert_eq!(lode.current_hazard, 1.0);
    }

    #[test]
    fn test_mining_increases_hazard_and_heat() {
        let mut world = World::new();
        // Setup Mother Lode entity
        let lode_entity = world.spawn((
            MotherLode {
                resource_type: ResourceType::Metal,
                current_hazard: 1.0,
                heat_output: 10.0,
            },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Setup resources
        world.insert_resource(ColonyResources::default());

        // Trigger mining (simulate one tick of mining action)
        // We assume a system handles the increment when mining occurs.
        // For the test, we call the system or helper directly.

        // Let's assume `mining_mother_lode_system` processes an event or state.
        // For MVP, we'll manually mutate to test the logic function `increment_hazard`.

        let mut lode = world.get_mut::<MotherLode>(lode_entity).unwrap();
        lode.increment_hazard();

        assert!(lode.current_hazard > 1.0, "Hazard should increase");
        assert!(lode.heat_output > 10.0, "Heat should increase");
    }

    #[test]
    fn test_heat_emission_into_grid() {
        let mut world = World::new();
        // Setup Grid
        let mut grid = TemperatureGrid::new(10, 10, 0.0);
        world.insert_resource(grid);
        world.insert_resource(crate::layer1::seasons::SeasonState::default());

        // Spawn Lode
        world.spawn((
            MotherLode {
                resource_type: ResourceType::Metal,
                current_hazard: 1.0,
                heat_output: 50.0, // Hot!
            },
            GridPosition { x: 5, y: 5 },
            // Need Building component if update_temperature_system relies on it,
            // OR update_temperature_system must query MotherLode specifically.
            // Let's assume we update the system to query MotherLode.
        ));

        // Run temperature update
        update_temperature_system(&mut world);

        let grid = world.resource::<TemperatureGrid>();
        assert!(grid.get(5, 5) >= 50.0, "Grid should receive heat from Lode");
    }

    #[test]
    fn test_hazard_impacts_risk_calculation() {
        // We need to modify calculate_risk to accept a hazard modifier
        let skills = Skills::default();
        let structure = Structure::default();
        let base_risk = 0.001;

        let risk_normal = calculate_risk(base_risk, &skills, SkillType::Mining, &structure, 1.0);
        let risk_mother_lode = calculate_risk(base_risk, &skills, SkillType::Mining, &structure, 10.0); // 10x hazard

        assert!(risk_mother_lode > risk_normal * 5.0, "High hazard should significantly increase risk");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `MotherLode` Component

Create `src/layer1/mother_lode.rs`:

```rust
use bevy_ecs::prelude::*;
use crate::layer1::resources::ResourceType;

#[derive(Component, Debug, Clone)]
pub struct MotherLode {
    pub resource_type: ResourceType,
    pub current_hazard: f32, // Multiplier for accident risk (starts at 1.0)
    pub heat_output: f32,    // Degrees added to local temp (starts at ~10.0)
}

impl MotherLode {
    pub fn increment_hazard(&mut self) {
        // Exponential or linear growth?
        // Linear is safer for gameplay balance.
        self.current_hazard += 0.1;
        self.heat_output += 2.0;
    }
}
```

### 2. Update Temperature System (`src/layer1/temperature.rs`)

Modify `update_temperature_system` to include `MotherLode` entities as heat sources.

```rust
use crate::layer1::mother_lode::MotherLode;

pub fn update_temperature_system(
    mut grid: ResMut<TemperatureGrid>,
    season: Res<SeasonState>,
    buildings: Query<(&Building, &GridPosition)>,
    lodes: Query<(&MotherLode, &GridPosition)>, // New query
) {
    // ... existing logic ...

    // Apply Lode Heat
    for (lode, pos) in &lodes {
        grid.add(pos.x, pos.y, lode.heat_output);
    }

    // ... diffusion ...
}
```

### 3. Update Risk Calculation (`src/layer1/hazards.rs`)

Modify `calculate_risk` signature to accept `hazard_modifier`.

```rust
pub fn calculate_risk(
    base_risk: f64,
    skills: &Skills,
    skill_type: SkillType,
    structure: &Structure,
    hazard_modifier: f64, // New param (default 1.0)
) -> f64 {
    // ... existing factors ...
    // Apply modifier
    let final_risk = (base_risk * maintenance_factor / skill_factor) * hazard_modifier;
    final_risk
}
```

**Note**: You will need to update existing calls to `calculate_risk` in `execution.rs` to pass `1.0` as the default modifier.

### 4. Implement Mining Logic Update

In `src/layer1/execution.rs` (or where `ActionType::Mine` is handled):

```rust
// Inside work execution loop
if let Some(mut lode) = world.get_mut::<MotherLode>(target_entity) {
    // 1. Give Resource (Standard amount)
    // 2. Do NOT deplete (Infinite)
    // 3. Increment Hazard
    lode.increment_hazard();

    // 4. Use high hazard for risk check
    let risk = calculate_risk(..., lode.current_hazard as f64);
    // ... trigger accident logic ...
} else {
    // Standard mining logic (depletes rock)
    let risk = calculate_risk(..., 1.0);
}
```

## REFACTOR Phase: Quality & Design

- **Visuals**: The Mother Lode should glow (use `131` Bioluminescence logic?). Render it with a bright color (Gold/Red) or pulsing animation.
- **UI**: When selecting the Lode, show a "Danger Level" bar.
- **Cooling**: Players can build `Heatsink` or `Cooler` buildings nearby to mitigate the heat, but the Accident Risk remains.
- **Events**: Trigger a "Tremor" event when Hazard crosses thresholds (10.0, 20.0).

## Acceptance Criteria

- [ ] `MotherLode` component exists.
- [ ] Mining the Lode yields resources without destroying the entity.
- [ ] Every mine operation increases `current_hazard` and `heat_output`.
- [ ] Lode emits heat into `TemperatureGrid`.
- [ ] Accident risk scales with `current_hazard`.
- [ ] Tests pass.

## Technical Guidance

- Ensure `calculate_risk` refactor doesn't break existing tests (update them to pass `1.0`).
- Use `SaturatingAdd` or cap the hazard if it gets too high (e.g., max 100.0) to prevent integer overflows or instant-death loops.
- `MotherLode` should be rare during map generation (Spec `095` or `002` update). For now, just spawning it manually or via debug command is fine.
