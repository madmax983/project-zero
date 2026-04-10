use crate::layer1::map::GridPosition;
use crate::layer1::structure::Structure;
use bevy_ecs::prelude::*;
use crate::layer1::building::Building;
use crate::layer1::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::events::BuildingRemovedEvent;


/// Component indicating a megastructure that consumes the structural integrity of nearby buildings.
#[derive(Component)]
pub struct ParasiticArchitecture {
    /// The radius within which buildings are consumed.
    pub radius: f32,
    /// The amount of structural integrity consumed per tick.
    pub consumption_rate: f32,
}

/// System that processes the consumption of buildings by parasitic megastructures.
pub fn process_megastructure_consumption(
    mut commands: Commands,
    megastructures: Query<(&GridPosition, &ParasiticArchitecture)>,
    mut buildings: Query<(Entity, &GridPosition, Option<&Building>, &mut Structure), Without<ParasiticArchitecture>>,
    mut removed_events: EventWriter<BuildingRemovedEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for (mega_pos, parasitic) in megastructures.iter() {
        let radius_sq = parasitic.radius * parasitic.radius;
        for (entity, build_pos, building, mut structure) in buildings.iter_mut() {
            let dx = (mega_pos.x as f32) - (build_pos.x as f32);
            let dy = (mega_pos.y as f32) - (build_pos.y as f32);
            let distance_sq = dx * dx + dy * dy;

            if distance_sq <= radius_sq {
                structure.current_hp -= parasitic.consumption_rate;

                if structure.current_hp <= 0.0 {
                    commands.entity(entity).despawn();
                    if let Some(building) = building {
                        removed_events.send(BuildingRemovedEvent {
                            entity,
                            position: *build_pos,
                            building_type: building.building_type,
                        });
                        chronicle_events.send(AddChronicleEvent {
                            text: format!("A {:?} was consumed by the parasitic megastructure at ({}, {}).", building.building_type, mega_pos.x, mega_pos.y),
                            importance: EventImportance::Standard,
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::structure::Structure;

    #[test]
    fn test_megastructure_consumes_nearby_building_integrity() {
        let mut world = World::new();
        world.init_resource::<Events<BuildingRemovedEvent>>();
        world.init_resource::<Events<AddChronicleEvent>>();

        let _megastructure = world
            .spawn((
                GridPosition { x: 10, y: 10 },
                Building {
                    building_type: BuildingType::Lander,
                },
                ParasiticArchitecture {
                    radius: 2.0,
                    consumption_rate: 10.0,
                },
            ))
            .id();

        let victim = world
            .spawn((
                GridPosition { x: 11, y: 10 },
                Building {
                    building_type: BuildingType::Housing,
                },
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
            ))
            .id();

        // Run the system once
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(process_megastructure_consumption);
        schedule.run(&mut world);

        // Assert the victim building lost integrity
        let structure = world.get::<Structure>(victim).unwrap();
        assert_eq!(structure.current_hp, 90.0);
    }

    #[test]
    fn test_megastructure_destroys_building() {
        let mut world = World::new();
        world.init_resource::<Events<BuildingRemovedEvent>>();
        world.init_resource::<Events<AddChronicleEvent>>();

        let _megastructure = world
            .spawn((
                GridPosition { x: 10, y: 10 },
                Building {
                    building_type: BuildingType::Lander,
                },
                ParasiticArchitecture {
                    radius: 2.0,
                    consumption_rate: 100.0,
                },
            ))
            .id();

        let victim = world
            .spawn((
                GridPosition { x: 11, y: 10 },
                Building {
                    building_type: BuildingType::Housing,
                },
                Structure {
                    current_hp: 50.0,
                    max_hp: 100.0,
                },
            ))
            .id();

        // Run the system once
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(process_megastructure_consumption);
        schedule.run(&mut world);

        // Assert the victim building was destroyed
        assert!(world.get::<Structure>(victim).is_none());
    }

    #[test]
    fn test_megastructure_ignores_out_of_range_buildings() {
        let mut world = World::new();
        world.init_resource::<Events<BuildingRemovedEvent>>();
        world.init_resource::<Events<AddChronicleEvent>>();

        let _megastructure = world
            .spawn((
                GridPosition { x: 10, y: 10 },
                Building {
                    building_type: BuildingType::Lander,
                },
                ParasiticArchitecture {
                    radius: 1.0,
                    consumption_rate: 10.0,
                },
            ))
            .id();

        let victim = world
            .spawn((
                GridPosition { x: 15, y: 15 },
                Building {
                    building_type: BuildingType::Housing,
                },
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
            ))
            .id();

        // Run the system once
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(process_megastructure_consumption);
        schedule.run(&mut world);

        // Assert the victim building is unaffected
        let structure = world.get::<Structure>(victim).unwrap();
        assert_eq!(structure.current_hp, 100.0);
    }
}
