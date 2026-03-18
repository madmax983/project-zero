# Spec 510: The Orbital Scavengers

## 1. Overview
Automated drone swarms deployed in orbit act as "Scavengers," reducing the risk of Orbital Debris Avalanches while periodically dropping valuable salvage onto the planet. These unguided "Salvage Pods" offer rare resources but carry a risk of damaging Layer 1 infrastructure upon impact.

## 2. Dependencies
- `184` Orbital Debris
- `152` Orbital Stations

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_scavenger_swarm_reduces_debris_density() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<OrbitalDebrisField>()
            .add_systems(Update, scavenger_drone_system);

        app.world_mut().resource_mut::<OrbitalDebrisField>().density = 50.0;

        let drone_entity = app.world_mut().spawn((
            ScavengerDrone { efficiency: 5.0, collected_mass: 0.0 },
        )).id();

        app.update();

        let field = app.world().resource::<OrbitalDebrisField>().density;
        assert!(field < 50.0, "Scavenger drone should reduce orbital debris density");
    }

    #[test]
    fn test_scavenger_accumulates_mass() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<OrbitalDebrisField>()
            .add_systems(Update, scavenger_drone_system);

        app.world_mut().resource_mut::<OrbitalDebrisField>().density = 50.0;

        let drone_entity = app.world_mut().spawn((
            ScavengerDrone { efficiency: 5.0, collected_mass: 0.0 },
        )).id();

        app.update();

        let mass = app.world().get::<ScavengerDrone>(drone_entity).unwrap().collected_mass;
        assert!(mass > 0.0, "Scavenger drone should accumulate mass from debris");
    }

    #[test]
    fn test_salvage_pod_drop_trigger() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_event::<SalvagePodDropEvent>()
            .init_resource::<OrbitalDebrisField>()
            .add_systems(Update, scavenger_drone_system);

        app.world_mut().resource_mut::<OrbitalDebrisField>().density = 50.0;

        let drone_entity = app.world_mut().spawn((
            ScavengerDrone { efficiency: 5.0, collected_mass: 98.0 }, // Near threshold
        )).id();

        app.update();

        let events = app.world().resource::<Events<SalvagePodDropEvent>>();
        assert_eq!(events.get_reader().len(&events), 1, "Scavenger drone should trigger a SalvagePodDropEvent upon reaching mass threshold");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct ScavengerDrone {
    pub efficiency: f32,
    pub collected_mass: f32,
}

#[derive(Event)]
pub struct SalvagePodDropEvent {
    pub value: f32,
    pub location: Vec3, // Calculated randomly or targeted later
}

#[derive(Resource, Default)]
pub struct OrbitalDebrisField {
    pub density: f32,
}

pub fn scavenger_drone_system(
    mut query: Query<&mut ScavengerDrone>,
    mut debris_field: ResMut<OrbitalDebrisField>,
    mut drop_events: EventWriter<SalvagePodDropEvent>,
) {
    let mass_threshold = 100.0;

    for mut drone in query.iter_mut() {
        if debris_field.density > 0.0 {
            let collected = drone.efficiency.min(debris_field.density);
            debris_field.density -= collected;
            drone.collected_mass += collected;

            if drone.collected_mass >= mass_threshold {
                drop_events.send(SalvagePodDropEvent {
                    value: drone.collected_mass,
                    location: Vec3::ZERO, // Placeholder for random Layer 1 location
                });
                drone.collected_mass -= mass_threshold;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create a `salvage_pod_impact_system` that handles the `SalvagePodDropEvent` to damage buildings at `location` and spawn an `ItemType::ScrapMetal` on the ground.
- The drop `location` shouldn't be `Vec3::ZERO`; calculate a random target on the surface grid during the drop event.

## 6. Acceptance Criteria
- [ ] Scavenger drones decrease `OrbitalDebrisField.density`.
- [ ] Scavenger drones accumulate `collected_mass`.
- [ ] Reaching a mass threshold triggers a `SalvagePodDropEvent`.
- [ ] Test coverage >= 85%.
- [ ] `cargo clippy -- -D warnings` passes.

## 7. Technical Guidance
- `OrbitalDebrisField` connects to the hazards defined in `184 Orbital Debris`. Ensure the drones provide a tangible benefit by delaying or preventing debris avalanches.
- The `SalvagePodDropEvent` acts as a bridge from Layer 2 (orbit) back to Layer 1 (surface).

## 8. Questions
*Builder: add questions here if spec is unclear.*
