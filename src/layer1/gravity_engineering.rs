use crate::layer1::events::BuildingCompletedEvent;
use crate::layer1::shields::DamageEvent;
use bevy_ecs::prelude::*;

/// Environment component indicating planetary gravity constraints.
#[derive(Resource)]
pub struct PlanetEnvironment {
    pub gravity: f32,
}

/// Structural height for gravity calculations.
#[derive(Component)]
pub struct Height {
    pub floors: u32,
}

/// Marker component for reinforced structural materials.
#[derive(Component)]
pub struct ReinforcedMaterial;

pub fn evaluate_structural_integrity_system(
    planet: Option<Res<PlanetEnvironment>>,
    mut construction_events: EventReader<BuildingCompletedEvent>,
    query: Query<(&Height, Option<&ReinforcedMaterial>)>,
    mut damage_events: EventWriter<DamageEvent>,
) {
    let Some(planet) = planet else {
        return;
    };

    // Inverse relationship: High gravity = lower max height
    let base_max_height = (10.0 / planet.gravity).floor() as u32;

    for event in construction_events.read() {
        if let Ok((height, reinforced)) = query.get(event.entity) {
            let is_reinforced = reinforced.is_some();
            if !is_reinforced && height.floors > base_max_height {
                // Building is too tall for this gravity
                damage_events.send(DamageEvent {
                    target: event.entity,
                    amount: 1000.0, // Catastrophic collapse for MVP
                    velocity: 0.0,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::architecture::{Building, BuildingType, Material, MaterialType};
    use bevy_app::App;
    use bevy_app::Update;

    #[test]
    fn test_building_exceeding_max_height_takes_structural_damage() {
        let mut app = App::new();
        app.insert_resource(PlanetEnvironment { gravity: 2.0 }); // High-G
        app.add_event::<BuildingCompletedEvent>();
        app.add_event::<DamageEvent>();
        app.add_systems(Update, evaluate_structural_integrity_system);

        let tall_building = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                Height { floors: 6 }, // Too tall for High-G standard materials (5 base max)
                Material(MaterialType::Wood), // standard material
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<BuildingCompletedEvent>>()
            .send(BuildingCompletedEvent {
                entity: tall_building,
            });

        app.update();

        let damage_events = app.world().resource::<Events<DamageEvent>>();
        let mut cursor = damage_events.get_cursor();
        let mut found = false;
        for event in cursor.read(damage_events) {
            if event.target == tall_building {
                found = true;
            }
        }

        assert!(
            found,
            "A building exceeding the gravity height limit should suffer structural damage."
        );
    }

    #[test]
    fn test_reinforced_materials_bypass_height_limits() {
        let mut app = App::new();
        app.insert_resource(PlanetEnvironment { gravity: 2.0 });
        app.add_event::<BuildingCompletedEvent>();
        app.add_event::<DamageEvent>();
        app.add_systems(Update, evaluate_structural_integrity_system);

        let reinforced_building = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                Height { floors: 10 },
                Material(MaterialType::Metal),
                ReinforcedMaterial,
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<BuildingCompletedEvent>>()
            .send(BuildingCompletedEvent {
                entity: reinforced_building,
            });

        app.update();

        let damage_events = app.world().resource::<Events<DamageEvent>>();
        assert_eq!(
            damage_events.get_cursor().len(damage_events),
            0,
            "Reinforced buildings should ignore gravity height limits."
        );
    }
}
