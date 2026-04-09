use bevy_ecs::prelude::*;

#[derive(Component, Default)]
/// Tracks the memories accumulated by a pop.
pub struct MemoryTracker {
    pub memories: Vec<PopMemoryType>,
}

impl MemoryTracker {
    pub fn has_memory(&self, memory: PopMemoryType) -> bool {
        self.memories.contains(&memory)
    }

    pub fn add_memory(&mut self, memory: PopMemoryType) {
        if !self.has_memory(memory.clone()) {
            self.memories.push(memory);
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
/// Represents the different types of memories a pop can have.
pub enum PopMemoryType {
    SurvivedFamine,
}

#[derive(Event)]
/// A significant event that pops can witness.
pub enum SignificantEvent {
    Famine,
}

#[derive(Event)]
/// Event representing a colony-wide food shortage.
pub struct FoodShortageEvent;

/// System that records significant events into pop memories.
pub fn record_significant_events_system(
    mut events: EventReader<SignificantEvent>,
    mut query: Query<&mut MemoryTracker>,
) {
    for event in events.read() {
        match event {
            SignificantEvent::Famine => {
                for mut tracker in query.iter_mut() {
                    tracker.add_memory(PopMemoryType::SurvivedFamine);
                }
            }
        }
    }
}

/// System that processes food shortage events and applies stress based on memories.
pub fn process_food_shortage_stress_system(
    mut events: EventReader<FoodShortageEvent>,
    mut query: Query<(&MemoryTracker, &mut crate::layer1::stress::StressTracker)>,
) {
    for _ in events.read() {
        for (tracker, mut stress) in query.iter_mut() {
            if tracker.has_memory(PopMemoryType::SurvivedFamine) {
                stress.accumulated_stress += 15.0; // > 10.0
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
        world.insert_resource(Events::<SignificantEvent>::default());

        let pop_entity = world.spawn((Pop, MemoryTracker::default())).id();

        // Emit a significant event
        world.send_event(SignificantEvent::Famine);

        let mut schedule = Schedule::default();
        schedule.add_systems(super::record_significant_events_system);
        schedule.run(&mut world);

        // Pop should now have a memory of the famine
        let tracker = world.get::<MemoryTracker>(pop_entity).unwrap();
        assert!(tracker.has_memory(PopMemoryType::SurvivedFamine));
    }

    #[test]
    fn test_memory_influences_stress_reaction() {
        let mut world = World::new();
        world.insert_resource(Events::<FoodShortageEvent>::default());

        let mut tracker = MemoryTracker::default();
        tracker.add_memory(PopMemoryType::SurvivedFamine);

        let pop_entity = world
            .spawn((
                Pop,
                StressTracker {
                    accumulated_stress: 0.0,
                },
                tracker,
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
