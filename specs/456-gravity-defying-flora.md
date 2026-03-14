# 456 - Gravity-Defying Flora (Helium-Vines)

## 1. Overview
This specification details the "Gravity-Defying Flora" feature for Layer 1. The feature introduces "Helium-Vines," a strange alien plant that pulls upward against gravity. These vines can be anchored to the ground or buildings to harvest their highly nutritious fruit. However, their upward pull creates a structural threat. If detached, they float away and are lost. If clustered too heavily on a weak roof, their collective upward pull can rip the roof off, exposing the building's contents to the weather. This introduces a tension between high-density, high-yield agriculture and managing the structural threat of negative weight on colony buildings.

## 2. Dependencies
- Layer 1 base architecture (grid, ticks, entities)
- `Building` and `StructuralIntegrity` systems (Specs 006, 338)
- `Farming` and `Flora` growth mechanics (Spec 008, 044, 059)
- `Weather` and `Storm` systems (Spec 079)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::building::{Building, StructuralIntegrity};
    use crate::layer1::flora::{Flora, FloraType};
    use crate::layer1::weather::{Weather, WeatherType};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            calculate_upward_pull_system,
            roof_detachment_system,
            float_away_system,
        ));
        app
    }

    #[test]
    fn test_helium_vine_exerts_upward_pull() {
        // Arrange
        let mut app = setup_app();

        let vine_entity = app.world_mut().spawn((
            Flora { flora_type: FloraType::HeliumVine },
            UpwardPull { force: 5.0 },
            AnchoredTo { target: Entity::PLACEHOLDER }, // Will be updated
        )).id();

        let building_entity = app.world_mut().spawn((
            Building,
            StructuralIntegrity { max_load: 100.0, current_load: 0.0, base_integrity: 50.0 },
            AccumulatedLift { total_force: 0.0 },
        )).id();

        app.world_mut().entity_mut(vine_entity).insert(AnchoredTo { target: building_entity });

        // Act
        app.update();

        // Assert
        let building_lift = app.world().get::<AccumulatedLift>(building_entity).unwrap();
        assert_eq!(building_lift.total_force, 5.0, "Building should accumulate 5.0 upward lift from the anchored vine.");
    }

    #[test]
    fn test_roof_detaches_when_lift_exceeds_integrity() {
        // Arrange
        let mut app = setup_app();

        let building_entity = app.world_mut().spawn((
            Building,
            StructuralIntegrity { max_load: 100.0, current_load: 0.0, base_integrity: 10.0 }, // Weak integrity
            AccumulatedLift { total_force: 15.0 }, // Lift > Integrity
            HasRoof(true),
        )).id();

        let vine_entity = app.world_mut().spawn((
            Flora { flora_type: FloraType::HeliumVine },
            UpwardPull { force: 15.0 },
            AnchoredTo { target: building_entity },
        )).id();

        // Act
        app.update();

        // Assert
        let has_roof = app.world().get::<HasRoof>(building_entity).unwrap();
        assert_eq!(has_roof.0, false, "Roof should be detached when lift exceeds structural integrity.");

        let anchored = app.world().get::<AnchoredTo>(vine_entity);
        assert!(anchored.is_none(), "Vine should no longer be anchored after roof detachment.");
        assert!(app.world().get::<FloatingAway>(vine_entity).is_some(), "Vine should be marked as floating away.");
    }

    #[test]
    fn test_storm_weakens_structural_integrity() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_weather_stress_system);

        app.world_mut().insert_resource(Weather { current: WeatherType::Storm });

        let building_entity = app.world_mut().spawn((
            Building,
            StructuralIntegrity { max_load: 100.0, current_load: 0.0, base_integrity: 20.0 },
            WeatherStress { multiplier: 1.0 },
        )).id();

        // Act
        app.update();

        // Assert
        let integrity = app.world().get::<StructuralIntegrity>(building_entity).unwrap();
        // A storm should temporarily reduce the effective structural integrity, or increase the load.
        // Assuming apply_weather_stress_system sets a modifier.
        let stress = app.world().get::<WeatherStress>(building_entity).unwrap();
        assert!(stress.multiplier > 1.0, "Storm should increase weather stress on the building.");
    }

    #[test]
    fn test_unanchored_vines_float_away_and_despawn() {
        // Arrange
        let mut app = setup_app();

        let vine_entity = app.world_mut().spawn((
            Flora { flora_type: FloraType::HeliumVine },
            UpwardPull { force: 5.0 },
            FloatingAway { timer: 10.0 }, // Despawns after 10 ticks/seconds
        )).id();

        // Act
        // Simulate time passing
        for _ in 0..10 {
            let mut floating = app.world_mut().get_mut::<FloatingAway>(vine_entity).unwrap();
            floating.timer -= 1.0;
            app.update();
        }

        // Assert
        assert!(app.world().get_entity(vine_entity).is_none(), "Floating vine should despawn after timer expires.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// Components
#[derive(Component)]
pub struct UpwardPull {
    pub force: f32,
}

#[derive(Component)]
pub struct AnchoredTo {
    pub target: Entity,
}

#[derive(Component)]
pub struct AccumulatedLift {
    pub total_force: f32,
}

#[derive(Component)]
pub struct HasRoof(pub bool);

#[derive(Component)]
pub struct FloatingAway {
    pub timer: f32,
}

#[derive(Component)]
pub struct WeatherStress {
    pub multiplier: f32,
}

// Systems
pub fn calculate_upward_pull_system(
    vines_query: Query<(&UpwardPull, &AnchoredTo)>,
    mut building_query: Query<&mut AccumulatedLift>,
) {
    // Reset accumulated lift
    for mut lift in building_query.iter_mut() {
        lift.total_force = 0.0;
    }

    // Accumulate pull from all anchored vines
    for (pull, anchored) in vines_query.iter() {
        if let Ok(mut lift) = building_query.get_mut(anchored.target) {
            lift.total_force += pull.force;
        }
    }
}

pub fn roof_detachment_system(
    mut commands: Commands,
    mut building_query: Query<(Entity, &AccumulatedLift, &crate::layer1::building::StructuralIntegrity, &mut HasRoof, Option<&WeatherStress>)>,
    mut vines_query: Query<(Entity, &AnchoredTo)>,
) {
    for (building_entity, lift, integrity, mut has_roof, weather_stress) in building_query.iter_mut() {
        if !has_roof.0 { continue; }

        let stress_mult = weather_stress.map_or(1.0, |ws| ws.multiplier);
        let effective_integrity = integrity.base_integrity / stress_mult;

        if lift.total_force > effective_integrity {
            // Roof detaches
            has_roof.0 = false;

            // Detach all vines anchored to this building
            for (vine_entity, anchored) in vines_query.iter_mut() {
                if anchored.target == building_entity {
                    commands.entity(vine_entity)
                        .remove::<AnchoredTo>()
                        .insert(FloatingAway { timer: 10.0 });
                }
            }
        }
    }
}

pub fn float_away_system(
    mut commands: Commands,
    mut floating_query: Query<(Entity, &mut FloatingAway)>,
) {
    for (entity, mut floating) in floating_query.iter_mut() {
        floating.timer -= 1.0;
        if floating.timer <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn apply_weather_stress_system(
    weather: Option<Res<crate::layer1::weather::Weather>>,
    mut stress_query: Query<&mut WeatherStress>,
) {
    if let Some(weather) = weather {
        let mult = match weather.current {
            crate::layer1::weather::WeatherType::Storm => 2.0,
            _ => 1.0,
        };
        for mut stress in stress_query.iter_mut() {
            stress.multiplier = mult;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Performance:** `calculate_upward_pull_system` currently iterates over all vines and then performs a lookup in `building_query`. If the number of vines is very large, this could become a bottleneck. Consider maintaining a list of connected entities on the building component to avoid looking up the building for every vine.
- **Visuals:** Add visual feedback (e.g., particles, visual detachment) when the roof detaches and when the vines start floating away.
- **Resource Loss:** Implement the logic to expose the stored resources to the weather when the roof is detached (e.g., accelerating `Spoilage`).
- **Events:** Emit a `RoofDetachedEvent` so other systems (like UI, chronicles, or audio) can react to the event.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes without warnings.
- [ ] Test coverage is ≥85% for the new `gravity_defying_flora.rs` file.
- [ ] Helium-Vines apply upward force to their anchored structure.
- [ ] Buildings lose their roof when upward force > structural integrity.
- [ ] Unanchored vines float away and despawn.
- [ ] Storms dynamically increase the chance of roof detachment.

## 7. Technical Guidance
- **System Ordering:** Ensure `apply_weather_stress_system` runs before `roof_detachment_system` to apply the correct modifiers. `calculate_upward_pull_system` must run before `roof_detachment_system`.
- **Integrity Calculation:** Be mindful of how `StructuralIntegrity` interacts with other loads (e.g., if heavy machinery is on the roof, does it counteract the pull?). For now, keep it simple: `upward_force > base_integrity`.
- **Location:** Create `src/layer1/flora/helium_vine.rs` or integrate into existing flora logic.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
