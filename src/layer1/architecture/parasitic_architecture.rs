//! Parasitic Architecture
//!
//! Megastructures that extract resources or structural integrity from nearby standard
//! buildings to fuel themselves or trigger specific events.

use crate::layer1::map::GridPosition;
use crate::layer1::structure::Structure;
use bevy_ecs::prelude::*;

/// Component indicating a megastructure that consumes the structural integrity of nearby buildings.
///
/// ## Examples
///
/// ```
/// use scale::layer1::architecture::parasitic_architecture::ParasiticArchitecture;
///
/// let mega = ParasiticArchitecture {
///     radius: 10.0,
///     consumption_rate: 5.0,
/// };
/// assert_eq!(mega.radius, 10.0);
/// ```
#[derive(Component)]
pub struct ParasiticArchitecture {
    /// The radius within which buildings are consumed.
    pub radius: f32,
    /// The amount of structural integrity consumed per tick.
    pub consumption_rate: f32,
}

/// Event triggered when a parasitic megastructure consumes a building entirely.
#[derive(Event, Debug)]
pub struct BuildingConsumedEvent {
    /// The entity of the building that was consumed.
    pub entity: Entity,
}

/// Drains HP from structures within the radius of a [`ParasiticArchitecture`] entity.
///
/// Buildings whose HP reaches zero are despawned, and a [`BuildingConsumedEvent`] is emitted.
pub fn process_megastructure_consumption(
    mut commands: Commands,
    mut events: EventWriter<BuildingConsumedEvent>,
    megastructures: Query<(&GridPosition, &ParasiticArchitecture)>,
    mut buildings: Query<(Entity, &GridPosition, &mut Structure), Without<ParasiticArchitecture>>,
) {
    for (mega_pos, parasitic) in megastructures.iter() {
        let radius_sq = parasitic.radius * parasitic.radius;
        for (entity, build_pos, mut structure) in buildings.iter_mut() {
            let dx = (mega_pos.x as f32) - (build_pos.x as f32);
            let dy = (mega_pos.y as f32) - (build_pos.y as f32);
            let distance_sq = dx * dx + dy * dy;

            if distance_sq <= radius_sq {
                structure.current_hp -= parasitic.consumption_rate;

                if structure.current_hp <= 0.0 {
                    events.send(BuildingConsumedEvent { entity });
                    commands.entity(entity).despawn();
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
        world.init_resource::<Events<BuildingConsumedEvent>>();
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
        world.init_resource::<Events<BuildingConsumedEvent>>();
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(process_megastructure_consumption);
        schedule.run(&mut world);

        // Assert the victim building was destroyed
        assert!(world.get::<Structure>(victim).is_none());
    }

    #[test]
    fn test_megastructure_ignores_out_of_range_buildings() {
        let mut world = World::new();

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
        world.init_resource::<Events<BuildingConsumedEvent>>();
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(process_megastructure_consumption);
        schedule.run(&mut world);

        // Assert the victim building is unaffected
        let structure = world.get::<Structure>(victim).unwrap();
        assert_eq!(structure.current_hp, 100.0);
    }
}
