use bevy_ecs::prelude::*;

#[derive(Component)]
/// Tracks if a pop has a memory of surviving famine.
pub struct SurvivedFamineMemory;

#[derive(Event)]
/// A significant event that pops can witness.
pub struct FamineEvent;

#[derive(Event)]
/// Event representing a colony-wide food shortage.
pub struct FoodShortageEvent;

/// System that records significant events into pop memories.
pub fn record_significant_events_system(
    mut commands: Commands,
    mut events: EventReader<FamineEvent>,
    query: Query<Entity, With<crate::layer1::pop::Pop>>,
) {
    for _ in events.read() {
        for entity in query.iter() {
            commands.entity(entity).try_insert(SurvivedFamineMemory);
        }
    }
}

/// System that processes food shortage events and applies stress based on memories.
pub fn process_food_shortage_stress_system(
    mut events: EventReader<FoodShortageEvent>,
    mut query: Query<(
        Option<&SurvivedFamineMemory>,
        &mut crate::layer1::stress::StressTracker,
    )>,
) {
    for _ in events.read() {
        for (memory, mut stress) in query.iter_mut() {
            if memory.is_some() {
                stress.accumulated_stress += 15.0;
            } else {
                stress.accumulated_stress += 5.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::stress::StressTracker;

    #[test]
    fn test_pop_gains_memory_from_event() {
        let mut world = World::new();
        // Setup events
        world.insert_resource(Events::<FamineEvent>::default());

        let pop_entity = world.spawn(Pop).id();

        // Emit a significant event
        world.send_event(FamineEvent);

        let mut schedule = Schedule::default();
        schedule.add_systems(super::record_significant_events_system);
        schedule.run(&mut world);

        // Pop should now have a memory of the famine
        assert!(world.get::<SurvivedFamineMemory>(pop_entity).is_some());
    }

    #[test]
    fn test_memory_influences_stress_reaction() {
        let mut world = World::new();
        world.insert_resource(Events::<FoodShortageEvent>::default());

        let pop_entity = world
            .spawn((
                Pop,
                StressTracker {
                    accumulated_stress: 0.0,
                },
                SurvivedFamineMemory,
            ))
            .id();

        // Simulate a minor food shortage
        world.send_event(FoodShortageEvent);

        let mut schedule = Schedule::default();
        schedule.add_systems(super::process_food_shortage_stress_system);
        schedule.run(&mut world);

        // Pop with famine memory should react with more stress
        let stress = world
            .get::<StressTracker>(pop_entity)
            .unwrap()
            .accumulated_stress;
        assert!(stress > 10.0); // Normal pop might only get 5.0 stress
    }
}
