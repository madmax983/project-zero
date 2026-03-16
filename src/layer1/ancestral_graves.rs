use crate::layer1::events::BuildingCompletedEvent;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::social::Relationships;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct AncestralGrave {
    pub pop_name: String,
    pub original_entity: Entity,
}

#[derive(Event, Debug, PartialEq)]
pub struct SacrilegeEvent {
    pub pos: GridPosition,
}

pub fn visit_grave_system(
    mut query: Query<(&GridPosition, &Relationships, &mut Needs), With<crate::layer1::pop::Pop>>,
    grave_query: Query<(&GridPosition, &AncestralGrave)>,
) {
    for (grave_pos, grave) in grave_query.iter() {
        for (pop_pos, relationships, mut needs) in query.iter_mut() {
            if pop_pos.distance_chebyshev(*grave_pos) <= 1 {
                // Check if they were related (affinity >= 80)
                if relationships.get_affinity(grave.original_entity) >= 80.0 {
                    needs.leisure = (needs.leisure + 0.1).clamp(0.0, 1.0);
                }
            }
        }
    }
}

pub fn build_system_wrapper(
    mut events: EventReader<BuildingCompletedEvent>,
    mut sacrilege_events: EventWriter<SacrilegeEvent>,
    buildings: Query<&GridPosition>,
    graves: Query<(Entity, &GridPosition), With<AncestralGrave>>,
    mut commands: Commands,
) {
    for event in events.read() {
        if let Ok(building_pos) = buildings.get(event.entity) {
            for (grave_entity, grave_pos) in graves.iter() {
                if building_pos.x == grave_pos.x && building_pos.y == grave_pos.y {
                    commands.entity(grave_entity).despawn();
                    sacrilege_events.send(SacrilegeEvent { pos: *building_pos });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::health::Dead;
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::{handle_pop_death_system, Pop, PopName};
    use crate::layer1::social::Relationships;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        world.init_resource::<Events<SacrilegeEvent>>();
        world.init_resource::<Events<crate::layer1::pop::PopDied>>();
        world.init_resource::<Events<BuildingCompletedEvent>>();
        world
    }

    #[test]
    fn test_grave_spawns_on_pop_death() {
        let mut world = setup_world();
        let pos = GridPosition { x: 5, y: 5 };

        let pop_entity = world
            .spawn((Pop, pos, PopName("Miner Bob".to_string()), Dead))
            .id();

        // Trigger death logic
        let _ = world.run_system_once(handle_pop_death_system);

        // Verify grave exists at the position
        let graves = world
            .query_filtered::<&GridPosition, With<AncestralGrave>>()
            .iter(&world)
            .collect::<Vec<_>>();
        assert_eq!(graves.len(), 1);
        assert_eq!(*graves[0], pos);

        let grave_comp = world.query::<&AncestralGrave>().single(&world);
        assert_eq!(grave_comp.pop_name, "Miner Bob");
        // Verify original_entity matches (in actual implementation we might just store Uuid, but spec uses Entity)
        assert_eq!(grave_comp.original_entity, pop_entity);
    }

    #[test]
    fn test_visiting_grave_grants_mood_buff_to_relative() {
        let mut world = setup_world();
        let pos = GridPosition { x: 5, y: 5 };

        // Dead kin entity
        let dead_kin = world.spawn(Pop).id();

        let relative_entity = world
            .spawn((
                Pop,
                pos, // Standing on grave
                Relationships::with_affinity(dead_kin, 85.0),
                Needs::default(),
            ))
            .id();

        // Setup grave with the relative's dead kin
        world.spawn((
            AncestralGrave {
                pop_name: "Miner Bob".to_string(),
                original_entity: dead_kin,
            },
            pos,
        ));

        // Run visit system
        let _ = world.run_system_once(visit_grave_system);

        // Verify buff applied
        let needs = world.get::<Needs>(relative_entity).unwrap();
        assert!(needs.leisure > 0.8, "Leisure was {}", needs.leisure); // Default is 0.8, expect an increase
    }

    #[test]
    fn test_building_over_grave_causes_sacrilege() {
        let mut world = setup_world();
        let pos = GridPosition { x: 5, y: 5 };

        world.spawn((
            AncestralGrave {
                pop_name: "Miner Bob".to_string(),
                original_entity: Entity::PLACEHOLDER,
            },
            pos,
        ));

        let building_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::Farm,
                },
                pos,
            ))
            .id();

        let mut events = world
            .get_resource_mut::<Events<BuildingCompletedEvent>>()
            .unwrap();
        events.send(BuildingCompletedEvent {
            entity: building_entity,
        });

        // Run wrapper system
        let _ = world.run_system_once(build_system_wrapper);

        // Global sacrilege penalty event should be emitted
        let events = world.resource::<Events<SacrilegeEvent>>();
        let mut cursor = events.get_cursor();
        assert_eq!(cursor.read(&events).len(), 1);
    }
}
