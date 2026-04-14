# 1006: The Terraforming Rejection

## 1. Overview
The Terraforming Rejection is a cross-layer mechanic that simulates the planet violently fighting back against attempts to tame it. Rapidly terraforming a world builds "Planetary Stress." If stress reaches a critical threshold, the planet triggers a massive autoimmune response—spawning catastrophic localized disasters (super-volcanoes, engineered mega-hurricanes) specifically targeting terraforming infrastructure. This forces players to choose a slow, careful approach over aggressively thawing or freezing an ecosystem.

## 2. Dependencies
- Layer 1 `AtmosphereGrid` and `PressureGrid` to simulate atmospheric/temperature changes.
- Layer 1 `Disaster` systems (e.g., localized storms, fissures).
- Layer 1 `Terraforming` structures or tasks that actively change tiles.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::terraforming::{TerraformEvent, PlanetaryStress};
    use crate::layer1::hazards::HazardFlora;
    use crate::layer1::terrain::GridPosition;
    use crate::layer1::disasters::DisasterEvent;

    #[test]
    fn test_rapid_terraforming_increases_planetary_stress() {
        let mut app = App::new();
        app.insert_resource(PlanetaryStress { value: 0.0, threshold: 100.0 });
        app.add_event::<TerraformEvent>();
        app.add_systems(Update, apply_terraforming_stress_system);

        // Send a massive terraform event
        app.world_mut().resource_mut::<Events<TerraformEvent>>().send(TerraformEvent {
            position: GridPosition { x: 5, y: 5, z: 0 },
            delta_temperature: 50.0, // Aggressive change
        });

        app.update();

        let stress = app.world().resource::<PlanetaryStress>();
        assert!(stress.value > 0.0, "Terraforming should increase planetary stress.");
    }

    #[test]
    fn test_critical_stress_triggers_disaster_at_terraform_site() {
        let mut app = App::new();
        app.insert_resource(PlanetaryStress { value: 95.0, threshold: 100.0 });
        app.add_event::<TerraformEvent>();
        app.add_event::<DisasterEvent>();
        app.add_systems(Update, (apply_terraforming_stress_system, trigger_autoimmune_response_system).chain());

        // Push stress over the threshold
        app.world_mut().resource_mut::<Events<TerraformEvent>>().send(TerraformEvent {
            position: GridPosition { x: 5, y: 5, z: 0 },
            delta_temperature: 10.0,
        });

        app.update();

        // Verify a disaster was spawned at the location
        let disaster_events = app.world().resource::<Events<DisasterEvent>>();
        let mut reader = disaster_events.get_reader();
        let mut found = false;
        for event in reader.read(disaster_events) {
            if event.position.x == 5 && event.position.y == 5 && event.position.z == 0 {
                found = true;
            }
        }

        assert!(found, "A disaster should trigger at the terraforming site when stress exceeds the threshold.");

        let stress = app.world().resource::<PlanetaryStress>();
        assert_eq!(stress.value, 0.0, "Planetary stress should reset after triggering a disaster.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer1/terraforming_rejection.rs
use bevy::prelude::*;
use crate::layer1::terraforming::TerraformEvent;
use crate::layer1::terrain::GridPosition;
use crate::layer1::disasters::{DisasterEvent, DisasterType};

#[derive(Resource)]
pub struct PlanetaryStress {
    pub value: f32,
    pub threshold: f32,
}

pub fn apply_terraforming_stress_system(
    mut stress: ResMut<PlanetaryStress>,
    mut events: EventReader<TerraformEvent>,
) {
    for event in events.read() {
        // Simple linear scaling for MVP
        stress.value += event.delta_temperature.abs() * 0.5;
    }
}

pub fn trigger_autoimmune_response_system(
    mut stress: ResMut<PlanetaryStress>,
    mut events: EventReader<TerraformEvent>,
    mut disaster_events: EventWriter<DisasterEvent>,
) {
    if stress.value >= stress.threshold {
        // Find the last known terraforming location to target
        if let Some(last_event) = events.read().last() {
            disaster_events.send(DisasterEvent {
                position: last_event.position.clone(),
                disaster_type: DisasterType::Fissure, // MVP: Just tear the ground open
            });
            // Reset stress after disaster
            stress.value = 0.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Stress Decay:** Planetary stress should naturally decay over time if terraforming is paused, allowing careful players to manage the mechanic.
- **Disaster Targeting:** Instead of just the *last* event, the response should target the tile or area with the highest *cumulative* terraforming change (the epicenter).
- **Disaster Types:** Differentiate the autoimmune response based on the current biome (e.g., Mega-Hurricanes on ocean worlds, Fissures on rocky worlds).

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_rapid_terraforming_increases_planetary_stress` passes.
- [ ] Test `test_critical_stress_triggers_disaster_at_terraform_site` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.

## 7. Technical Guidance
- The system must hook into however `TerraformEvent` or its equivalent is fired in the game engine.
- Ensure that the order of operations increments the stress *before* checking the threshold within the same frame or update tick.

## 8. Questions
*Builder: add questions here if spec is unclear.*
