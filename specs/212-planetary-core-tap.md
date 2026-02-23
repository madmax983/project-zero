# 212: Planetary Core Tap

## Overview

The **Planetary Core Tap** is a high-risk, high-reward endgame structure. It bypasses fuel logistics and solar cycles by drilling directly into the planet's mantle for infinite geothermal energy. However, this process destabilizes the crust, generating constant **Seismic Stress** (Spec 153). If not managed (via pausing the drill or spreading out taps), it triggers massive earthquakes that can destroy the colony.

## Dependencies

- `042` — Energy System (for `PowerSource`)
- `153` — Geological Instability (for `SeismicGrid`)
- `004` — Building System (for the structure)

## RED Phase: Tests First

Write these tests in `src/layer1/core_tap_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::energy::PowerSource;
    use crate::layer1::geology::{SeismicGrid, GeologicalEvent, add_seismic_stress};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::core_tap::{CoreTap, update_core_tap_system};

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(SeismicGrid::new(10, 10));
        world
    }

    #[test]
    fn test_core_tap_building_properties() {
        let b = BuildingType::CoreTap;
        assert_eq!(b.label(), "Core Tap");
        assert!(b.cost().metal > 50.0); // Expensive
    }

    #[test]
    fn test_core_tap_generates_massive_power() {
        let mut world = setup_world();
        let tap = world.spawn((
            Building { building_type: BuildingType::CoreTap },
            CoreTap { stress_per_tick: 0.5 },
            PowerSource { output: 500.0, active: true }, // Massive output
            GridPosition { x: 5, y: 5 },
        )).id();

        // Check power output is maintained
        let source = world.get::<PowerSource>(tap).unwrap();
        assert_eq!(source.output, 500.0);
    }

    #[test]
    fn test_core_tap_generates_seismic_stress() {
        let mut world = setup_world();
        let tap = world.spawn((
            Building { building_type: BuildingType::CoreTap },
            CoreTap { stress_per_tick: 5.0 },
            PowerSource { output: 100.0, active: true },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_core_tap_system);
        schedule.run(&mut world);

        // Check grid stress
        let grid = world.resource::<SeismicGrid>();
        // Stress should be added at position (5,5)
        assert!(grid.get_stress(5, 5) >= 5.0);
    }

    #[test]
    fn test_core_tap_stops_stress_when_inactive() {
        let mut world = setup_world();
        let tap = world.spawn((
            Building { building_type: BuildingType::CoreTap },
            CoreTap { stress_per_tick: 5.0 },
            PowerSource { output: 100.0, active: false }, // Inactive
            GridPosition { x: 5, y: 5 },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_core_tap_system);
        schedule.run(&mut world);

        let grid = world.resource::<SeismicGrid>();
        assert_eq!(grid.get_stress(5, 5), 0.0);
    }

    #[test]
    fn test_multiple_taps_compound_stress() {
        let mut world = setup_world();
        // Tap 1 at (5,5)
        world.spawn((
            Building { building_type: BuildingType::CoreTap },
            CoreTap { stress_per_tick: 5.0 },
            PowerSource { output: 100.0, active: true },
            GridPosition { x: 5, y: 5 },
        ));
        // Tap 2 at (5,5) (Stacking on same tile, technically possible in ECS logic if not placement logic)
        world.spawn((
            Building { building_type: BuildingType::CoreTap },
            CoreTap { stress_per_tick: 5.0 },
            PowerSource { output: 100.0, active: true },
            GridPosition { x: 5, y: 5 },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_core_tap_system);
        schedule.run(&mut world);

        let grid = world.resource::<SeismicGrid>();
        // Should be 10.0 total
        assert!(grid.get_stress(5, 5) >= 10.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `BuildingType`

Add `CoreTap` to `src/layer1/building.rs`.
- Cost: Metal 200.0, Stone 100.0 (Expensive).
- Char: `O` or `⦿` (Red).

### 2. Define Component (`src/layer1/core_tap.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::energy::PowerSource;
use crate::layer1::map::GridPosition;
use crate::layer1::geology::add_seismic_stress;

#[derive(Component, Debug, Clone, Copy)]
pub struct CoreTap {
    pub stress_per_tick: f32,
}

impl Default for CoreTap {
    fn default() -> Self {
        Self { stress_per_tick: 0.5 } // Tunable
    }
}

pub fn update_core_tap_system(
    mut world: &mut World,
    // Note: We use World access because add_seismic_stress needs World.
    // Alternatively, inject ResMut<SeismicGrid> and Query.
) {
    // Better signature for ECS safety:
    // fn update_core_tap_system(
    //    mut grid: ResMut<SeismicGrid>,
    //    query: Query<(&CoreTap, &PowerSource, &GridPosition)>,
    // )
}

// Actual implementation
pub fn update_core_tap_system(
    mut grid: ResMut<crate::layer1::geology::SeismicGrid>,
    query: Query<(&CoreTap, &PowerSource, &GridPosition)>,
) {
    for (tap, source, pos) in query.iter() {
        if source.active {
            grid.add_stress(pos.x, pos.y, tap.stress_per_tick);
        }
    }
}
```

### 3. Integration

- Register `CoreTap` component.
- Register `update_core_tap_system` in `SimulationSchedule`, after `power_grid_system` (to check active status) but before `check_seismic_events`.

## REFACTOR Phase: Quality & Design

- **Heat Output**: A Core Tap should generate massive Heat (Spec 140). Add `HeatSource` component.
- **Visuals**: Add screen shake or rumble sound when near active taps.
- **UI Warning**: If local stress > 80%, flash a warning on the building UI.
- **Upgrades**: Allow researching "Stabilizers" to reduce stress per tick.

## Acceptance Criteria

- [ ] `CoreTap` building is constructible.
- [ ] Active `CoreTap` generates power (via `PowerSource`).
- [ ] Active `CoreTap` increases `SeismicGrid` stress at its location.
- [ ] Inactive `CoreTap` generates no stress.
- [ ] High stress eventually triggers earthquakes (via existing Spec 153 logic).
- [ ] Tests pass.

## Technical Guidance

- Ensure `CoreTap`'s `PowerSource.output` is high enough to justify the risk (e.g., 500.0 vs Solar 10.0).
- Check `SeismicGrid` decay rate. `CoreTap` accumulation must be faster than decay for risk to accumulate, or require multiple taps to trigger quakes.
- Default `stress_per_tick` should be balanced against `DECAY_RATE`. If decay is 0.5, stress must be > 0.5 to accumulate.
