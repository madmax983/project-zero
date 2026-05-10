use crate::layer1::events::BuildingCompletedEvent;
use crate::layer1::funeral::Grave;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use bevy_ecs::prelude::*;

#[derive(Event, Debug, PartialEq)]
pub struct SacrilegeEvent {
    pub pos: GridPosition,
}

pub fn grave_visit_system(
    mut pops: Query<(&GridPosition, &mut Needs), With<Pop>>,
    graves: Query<&GridPosition, With<Grave>>,
) {
    for (pop_pos, mut needs) in pops.iter_mut() {
        for grave_pos in graves.iter() {
            if pop_pos.x.abs_diff(grave_pos.x) <= 1 && pop_pos.y.abs_diff(grave_pos.y) <= 1 {
                needs.leisure = (needs.leisure + 0.1).min(1.0);
            }
        }
    }
}

pub fn build_system_wrapper(
    mut events: EventReader<BuildingCompletedEvent>,
    mut sacrilege_events: EventWriter<SacrilegeEvent>,
    buildings: Query<&GridPosition>,
    graves: Query<&GridPosition, With<Grave>>,
) {
    for event in events.read() {
        if let Ok(building_pos) = buildings.get(event.entity) {
            for grave_pos in graves.iter() {
                if building_pos.x == grave_pos.x && building_pos.y == grave_pos.y {
                    sacrilege_events.send(SacrilegeEvent { pos: *building_pos });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_grave_provides_mood_buff_to_visitor() {
        // Arrange
        let mut world = World::new();
        world.spawn((Grave::default(), GridPosition { x: 0, y: 0 }));
        let visitor = world
            .spawn((
                Pop,
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
                GridPosition { x: 1, y: 0 },
            ))
            .id();

        // Act
        let _ = world.run_system_once(grave_visit_system);

        // Assert
        let needs = world.get::<Needs>(visitor).unwrap();
        assert!(
            needs.leisure > 0.5,
            "Visiting grave should restore leisure/mood"
        );
    }

    #[test]
    fn test_building_over_grave_causes_sacrilege() {
        // Arrange
        let mut world = World::new();
        let grave_pos = GridPosition { x: 0, y: 0 };
        world.spawn((Grave::default(), grave_pos));
        let building_entity = world.spawn(grave_pos).id();
        let mut events = Events::<BuildingCompletedEvent>::default();
        events.send(BuildingCompletedEvent {
            entity: building_entity,
        });
        world.insert_resource(events);
        world.insert_resource(Events::<SacrilegeEvent>::default());

        // Act
        let _ = world.run_system_once(build_system_wrapper);

        // Assert
        let sacrilege_events = world.get_resource::<Events<SacrilegeEvent>>().unwrap();
        assert_eq!(
            sacrilege_events.len(),
            1,
            "Building over a grave should trigger sacrilege"
        );
    }
}
