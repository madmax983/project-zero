use crate::layer1::architecture::building::{Height, Material};
use crate::layer1::architecture::structure::Structure;
use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::core::events::BuildingCompletedEvent;
use crate::layer2::syzygy::PlanetaryGravity;
use bevy::prelude::*;

pub fn evaluate_structural_integrity_system(
    gravity: Option<Res<PlanetaryGravity>>,
    mut construction_events: EventReader<BuildingCompletedEvent>,
    mut query: Query<(&mut Height, &Material, &mut Structure)>,
) {
    let g = gravity.map(|g| g.current).unwrap_or(1.0);
    // Inverse relationship: High gravity = lower max height
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    let base_max_height = (10.0 / g).floor() as u32;

    for event in construction_events.read() {
        if let Ok((mut height, material, mut structure)) = query.get_mut(event.entity) {
            if !material.0.is_reinforced() && height.floors > base_max_height {
                // Progressive Collapse: deduct 50.0 HP per excess floor instead of instant total destruction
                let excess_floors = height.floors.saturating_sub(base_max_height);
                height.floors = base_max_height;

                #[allow(clippy::cast_precision_loss)]
                let damage = (excess_floors as f32) * 50.0;

                structure.current_hp -= damage;
                if structure.current_hp < 0.0 {
                    structure.current_hp = 0.0;
                }
            }
        }
    }
}

pub fn gravity_engineering_chronicle_bridge(
    mut events: EventReader<crate::layer1::core::events::BuildingCompletedEvent>,
    query: Query<(
        &crate::layer1::architecture::building::Height,
        &crate::layer1::architecture::structure::Structure,
    )>,
    mut chronicle: EventWriter<AddChronicleEvent>,
) {
    for event in events.read() {
        if let Ok((height, structure)) = query.get(event.entity) {
            if structure.current_hp < structure.max_hp && structure.current_hp > 0.0 {
                chronicle.send(AddChronicleEvent {
                    text: format!("Our newly constructed building ({} floors) suffered a partial structural collapse under its own weight due to the local gravity!", height.floors),
                    importance: EventImportance::Major,
                });
            } else if structure.current_hp == 0.0 {
                chronicle.send(AddChronicleEvent {
                    text: format!("Our newly constructed building ({} floors) instantly collapsed under the crushing gravity!", height.floors),
                    importance: EventImportance::Major,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::architecture::building::{Building, MaterialType};

    #[test]
    fn test_building_exceeding_max_height_takes_structural_damage() {
        let mut app = App::new();
        app.insert_resource(PlanetaryGravity {
            current: 2.0,
            base: 2.0,
        }); // High-G
        app.add_event::<BuildingCompletedEvent>();
        app.add_systems(Update, evaluate_structural_integrity_system);

        let tall_building = app
            .world_mut()
            .spawn((
                Building {
                    building_type: crate::layer1::architecture::building::BuildingType::Housing,
                },
                Height { floors: 6 }, // Too tall for High-G standard materials
                Material(MaterialType::Wood), // not reinforced
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<BuildingCompletedEvent>>()
            .send(BuildingCompletedEvent {
                entity: tall_building,
            });

        app.update();

        let structure = app.world().get::<Structure>(tall_building).unwrap();
        assert!(
            structure.current_hp < 100.0,
            "A building exceeding the gravity height limit should suffer structural damage."
        );
    }

    #[test]
    fn test_reinforced_materials_bypass_height_limits() {
        let mut app = App::new();
        app.insert_resource(PlanetaryGravity {
            current: 2.0,
            base: 2.0,
        });
        app.add_event::<BuildingCompletedEvent>();
        app.add_systems(Update, evaluate_structural_integrity_system);

        let reinforced_building = app
            .world_mut()
            .spawn((
                Building {
                    building_type: crate::layer1::architecture::building::BuildingType::Housing,
                },
                Height { floors: 10 },
                Material(MaterialType::Metal), // reinforced
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<BuildingCompletedEvent>>()
            .send(BuildingCompletedEvent {
                entity: reinforced_building,
            });

        app.update();

        let structure = app.world().get::<Structure>(reinforced_building).unwrap();
        assert_eq!(
            structure.current_hp, 100.0,
            "Reinforced buildings should ignore gravity height limits."
        );
    }
}
