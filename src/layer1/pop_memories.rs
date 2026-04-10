use bevy_ecs::prelude::*;
use crate::layer1::stress::StressTracker;

#[derive(Component, Default)]
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
pub enum PopMemoryType {
    SurvivedFamine,
}

#[derive(Event, Clone, Debug, PartialEq)]
pub enum SignificantEvent {
    Famine,
}

#[derive(Event, Clone, Debug)]
pub struct FoodShortageEvent;

pub fn record_significant_events_system(
    mut events: EventReader<SignificantEvent>,
    mut query: Query<&mut MemoryTracker>,
) {
    for event in events.read() {
        if event == &SignificantEvent::Famine {
            for mut tracker in query.iter_mut() {
                tracker.add_memory(PopMemoryType::SurvivedFamine);
            }
        }
    }
}

pub fn process_food_shortage_event_system(
    mut events: EventReader<FoodShortageEvent>,
    mut query: Query<(&MemoryTracker, &mut StressTracker)>,
) {
    for _ in events.read() {
        for (tracker, mut stress) in query.iter_mut() {
            if tracker.has_memory(PopMemoryType::SurvivedFamine) {
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
    use bevy_app::prelude::App;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::stress::StressTracker;

    #[test]
    fn test_pop_gains_memory_from_event() {
        let mut app = App::new();
        // Setup systems and event streams
        app.add_event::<SignificantEvent>();
        app.add_systems(bevy_app::Update, record_significant_events_system);

        let pop_entity = app.world_mut().spawn((Pop, MemoryTracker::default())).id();

        // Emit a significant event
        app.world_mut().send_event(SignificantEvent::Famine);
        app.update();

        // Pop should now have a memory of the famine
        let tracker = app.world().get::<MemoryTracker>(pop_entity).unwrap();
        assert!(tracker.has_memory(PopMemoryType::SurvivedFamine));
    }

    #[test]
    fn test_memory_influences_stress_reaction() {
        let mut app = App::new();
        // Setup systems
        app.add_event::<FoodShortageEvent>();
        app.add_systems(bevy_app::Update, process_food_shortage_event_system);

        let mut tracker = MemoryTracker::default();
        tracker.add_memory(PopMemoryType::SurvivedFamine);

        let pop_entity = app.world_mut().spawn((
            Pop,
            StressTracker { accumulated_stress: 0.0 },
            tracker
        )).id();

        // Simulate a minor food shortage
        app.world_mut().send_event(FoodShortageEvent);
        app.update();

        // Pop with famine memory should react with more stress
        let stress = app.world().get::<StressTracker>(pop_entity).unwrap().accumulated_stress;
        assert!(stress > 10.0); // Normal pop might only get 5.0 stress
    }
}
