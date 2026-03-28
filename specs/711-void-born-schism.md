# 711: The Void-Born Schism

## Overview

A massive cultural divergence occurs when quality of life differs wildly between planet-bound Pops ("Dirt-Dwellers") and those stationed in space ("Void-Born"). Pops spending extended time in zero-G or on stations develop a "Void-Born" trait. Over time, as this trait deepens, they view planetary pops with increasing contempt. If this disparity grows too large, a schism occurs, causing the station or fleet to embargo the surface and potentially declare independence.

## Dependencies

- Requires an existing way to track where a Pop is located (Planet vs Orbital Station / Fleet) from Layer 2.

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::social::morale::Morale;

    #[test]
    fn test_void_born_trait_accumulation() {
        let mut app = App::new();
        app.add_systems(Update, accumulate_void_born_system);

        let pop = app.world_mut().spawn((
            OrbitalStationResident,
            VoidExposureTracker { time_in_space: 0.0 },
        )).id();

        // Act
        app.update();

        // Assert: Time tracking increased
        let tracker = app.world().get::<VoidExposureTracker>(pop).unwrap();
        assert!(tracker.time_in_space > 0.0);

        // Arrange: High exposure
        app.world_mut().entity_mut(pop).get_mut::<VoidExposureTracker>().unwrap().time_in_space = 1000.0;
        app.update();

        // Assert: VoidBornTrait added
        assert!(app.world().get::<VoidBornTrait>(pop).is_some());
    }

    #[test]
    fn test_schism_trigger_due_to_disparity() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_schism_system);

        let station = app.world_mut().spawn((
            StationCore,
            AverageStationMorale { value: 95.0 }, // Very comfortable
            PlanetaryMoraleTracker { planetary_value: 20.0 }, // The dirt dwellers are suffering
            VoidBornPopulationCount { count: 50 }, // Critical mass
        )).id();

        // Act
        app.update();

        // Assert: Station declares schism
        assert!(app.world().get::<SchismState>(station).is_some());
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct OrbitalStationResident;

#[derive(Component)]
pub struct VoidExposureTracker {
    pub time_in_space: f32,
}

#[derive(Component)]
pub struct VoidBornTrait;

#[derive(Component)]
pub struct StationCore;

#[derive(Component)]
pub struct AverageStationMorale {
    pub value: f32,
}

#[derive(Component)]
pub struct PlanetaryMoraleTracker {
    pub planetary_value: f32,
}

#[derive(Component)]
pub struct VoidBornPopulationCount {
    pub count: usize,
}

#[derive(Component)]
pub struct SchismState;

pub fn accumulate_void_born_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut VoidExposureTracker), With<OrbitalStationResident>>,
) {
    for (entity, mut tracker) in query.iter_mut() {
        tracker.time_in_space += 1.0; // Assume 1 unit per tick

        if tracker.time_in_space > 500.0 {
            commands.entity(entity).insert(VoidBornTrait);
        }
    }
}

pub fn evaluate_schism_system(
    mut commands: Commands,
    query: Query<(Entity, &AverageStationMorale, &PlanetaryMoraleTracker, &VoidBornPopulationCount), With<StationCore>>,
) {
    for (entity, station_morale, planet_morale, pop_count) in query.iter() {
        // High station morale vs low planetary morale
        let disparity = station_morale.value - planet_morale.value;

        // If the gap is huge and the station has enough Void-Born...
        if disparity > 50.0 && pop_count.count > 20 {
            commands.entity(entity).insert(SchismState);
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Architecture**: `AverageStationMorale`, `PlanetaryMoraleTracker`, and `VoidBornPopulationCount` should ideally be derived values from an aggregator system querying the `Morale` of Pops associated with that station/planet, rather than raw structural components holding values directly.
- **Economic Integration**: The `SchismState` component needs a system that intercepts `TradeRoute` or `HaulingTask` logic and rejects anything targeting the planetary surface.
- **Event Flow**: Firing a `SchismEvent` for integration with the Chronicle is mandatory.

## Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops residing on orbital stations accumulate space-time and gain the VoidBornTrait.
- [ ] A station with high morale and high Void-Born population relative to a struggling planet triggers a `SchismState`.

## Technical Guidance
- Implement this module in `src/layer2/social/void_schism.rs` as it bridges Layer 1 Pop traits with Layer 2 structures.
- Ensure the ECS structure aligns with your current station and planet representations.
- This creates an excellent opportunity for a future integration spec: "Integration: Void Schism -> Trade Disruption".

## Questions
*Builder: add questions here if spec is unclear.*
