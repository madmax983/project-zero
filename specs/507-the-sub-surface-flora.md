# Spec 507: The Sub-Surface Flora

## 1. Overview
The colony discovers a deep subterranean ecosystem of "Tectonic Roots" that thrive on tectonic energy. These plants provide a high-yield food source but farming them destabilizes the local crust, increasing the frequency of minor tremors and potentially swallowing infrastructure.

## 2. Dependencies
- `008` Farm (for basic crop mechanics)
- `252` Tectonic Stress (for seismic interactions)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_tectonic_root_growth_accelerated_by_stress() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<GlobalTectonicStress>()
            .add_systems(Update, grow_tectonic_roots_system);

        let root_entity = app.world_mut().spawn((
            TectonicRoot { growth_progress: 0.0, base_growth_rate: 1.0 },
        )).id();

        // High stress accelerates growth
        app.world_mut().resource_mut::<GlobalTectonicStress>().level = 80.0;
        app.update();

        let growth = app.world().get::<TectonicRoot>(root_entity).unwrap().growth_progress;
        assert!(growth > 1.0, "Growth should be accelerated by high tectonic stress");
    }

    #[test]
    fn test_harvesting_roots_increases_local_instability() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_event::<HarvestCropEvent>()
            .init_resource::<GlobalTectonicStress>()
            .add_systems(Update, harvest_tectonic_roots_system);

        let farm_entity = app.world_mut().spawn(Transform::from_xyz(10.0, 0.0, 10.0)).id();
        let root_entity = app.world_mut().spawn((
            TectonicRoot { growth_progress: 100.0, base_growth_rate: 1.0 },
            Parent(farm_entity),
        )).id();

        app.world_mut().send_event(HarvestCropEvent { crop: root_entity });

        let initial_stress = app.world().resource::<GlobalTectonicStress>().level;
        app.update();

        let final_stress = app.world().resource::<GlobalTectonicStress>().level;
        assert!(final_stress > initial_stress, "Harvesting tectonic roots should increase global tectonic stress");
    }

    #[test]
    fn test_excessive_harvesting_triggers_localized_tremor() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_event::<HarvestCropEvent>()
            .add_event::<LocalizedTremorEvent>()
            .init_resource::<GlobalTectonicStress>()
            .add_systems(Update, harvest_tectonic_roots_system);

        let farm_entity = app.world_mut().spawn(Transform::from_xyz(10.0, 0.0, 10.0)).id();
        let root_entity = app.world_mut().spawn((
            TectonicRoot { growth_progress: 100.0, base_growth_rate: 1.0 },
            Parent(farm_entity),
        )).id();

        // Push stress near the threshold
        app.world_mut().resource_mut::<GlobalTectonicStress>().level = 95.0;

        app.world_mut().send_event(HarvestCropEvent { crop: root_entity });
        app.update();

        let tremor_events = app.world().resource::<Events<LocalizedTremorEvent>>();
        assert_eq!(tremor_events.get_reader().len(&tremor_events), 1, "Excessive harvesting should trigger a localized tremor event");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct TectonicRoot {
    pub growth_progress: f32,
    pub base_growth_rate: f32,
}

#[derive(Event)]
pub struct HarvestCropEvent {
    pub crop: Entity,
}

#[derive(Event)]
pub struct LocalizedTremorEvent {
    pub location: Vec3,
    pub magnitude: f32,
}

#[derive(Resource, Default)]
pub struct GlobalTectonicStress {
    pub level: f32,
}

pub fn grow_tectonic_roots_system(
    mut query: Query<&mut TectonicRoot>,
    stress: Res<GlobalTectonicStress>,
) {
    let stress_multiplier = 1.0 + (stress.level / 100.0);
    for mut root in query.iter_mut() {
        root.growth_progress += root.base_growth_rate * stress_multiplier;
    }
}

pub fn harvest_tectonic_roots_system(
    mut harvest_events: EventReader<HarvestCropEvent>,
    mut stress: ResMut<GlobalTectonicStress>,
    mut tremor_events: EventWriter<LocalizedTremorEvent>,
    roots: Query<&Parent, With<TectonicRoot>>,
    transforms: Query<&Transform>,
) {
    for event in harvest_events.read() {
        if let Ok(parent) = roots.get(event.crop) {
            if let Ok(transform) = transforms.get(parent.get()) {
                stress.level += 5.0; // Increase stress slightly per harvest

                if stress.level >= 100.0 {
                    tremor_events.send(LocalizedTremorEvent {
                        location: transform.translation,
                        magnitude: stress.level / 10.0,
                    });
                    stress.level -= 50.0; // Release some stress after the tremor
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Consider moving `LocalizedTremorEvent` into the `tectonic_stress` module if it's reused elsewhere.
- Add a component to track local seismic instability per-tile, rather than purely relying on `GlobalTectonicStress`, for more precise localized tremors.
- Tweak the growth multiplier and stress increase values for balance.

## 6. Acceptance Criteria
- [ ] Tectonic roots grow faster when global tectonic stress is high.
- [ ] Harvesting tectonic roots increases tectonic stress.
- [ ] Reaching maximum tectonic stress triggers a localized tremor event near the harvest location.
- [ ] Test coverage >= 85%.
- [ ] `cargo clippy -- -D warnings` passes.

## 7. Technical Guidance
- `TectonicRoot` should probably be an `ItemType` in the `ColonyInventory` and integrated into the global food system after harvest.
- Ensure that `LocalizedTremorEvent` is caught by the chronicle bridge to generate lore entries about farming-induced earthquakes.

## 8. Questions
*Builder: add questions here if spec is unclear.*
