use bevy_ecs::prelude::*;
use crate::layer1::pop::Pop;
use crate::layer1::psychology::memory::{Memories, MemoryType};
use crate::layer1::psychology::stress::StressTracker;
use crate::layer1::social::unrest::{MentalState, MentalBreakType};

#[derive(Component)]
pub struct MnesticArchiver;

#[derive(Component, Default)]
pub struct ForgedMemories {
    pub forged: bool,
}

#[derive(Event)]
pub struct ForgeryActionEvent {
    pub pop: Entity,
}

#[derive(Event)]
pub struct TruthOutbreakEvent;

pub fn process_mnestic_archiver_system(
    mut events: EventReader<ForgeryActionEvent>,
    mut query: Query<(&mut Memories, &mut StressTracker, &mut ForgedMemories), With<Pop>>,
) {
    for event in events.read() {
        if let Ok((mut memories, mut stress, mut forged)) = query.get_mut(event.pop) {
            // Remove negative memories
            memories.items.retain(|m| {
                m.memory_type.base_mood_impact() >= 0.0
            });
            // Add a fake positive memory
            memories.add(MemoryType::FakePositive, 0);
            stress.accumulated_stress = 0.0;
            forged.forged = true;
        }
    }
}

pub fn process_truth_outbreak_system(
    mut events: EventReader<TruthOutbreakEvent>,
    mut query: Query<(&ForgedMemories, &mut MentalState), With<Pop>>,
) {
    for _event in events.read() {
        for (forged, mut mental_state) in query.iter_mut() {
            if forged.forged {
                // Trigger Reality Collapse by making them Vandalize
                *mental_state = MentalState::Broken(MentalBreakType::Vandalize);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

    #[test]
    fn test_mnestic_archiver_erases_bad_memories() {
        let mut app = App::new();
        app.add_event::<ForgeryActionEvent>();
        app.add_systems(Update, process_mnestic_archiver_system);

        let mut memories = Memories::default();
        memories.add(MemoryType::WitnessedDeath, 0); // Negative
        let pop = app.world_mut().spawn((
            Pop,
            memories,
            StressTracker { accumulated_stress: 50.0 },
            ForgedMemories::default(),
        )).id();

        app.world_mut().send_event(ForgeryActionEvent { pop });
        app.update();

        let updated_memories = app.world().get::<Memories>(pop).unwrap();
        let forged = app.world().get::<ForgedMemories>(pop).unwrap();
        assert!(!updated_memories.items.iter().any(|m| m.memory_type == MemoryType::WitnessedDeath));
        assert!(updated_memories.items.iter().any(|m| m.memory_type == MemoryType::FakePositive));
        assert!(forged.forged);
    }

    #[test]
    fn test_mnestic_archiver_boosts_morale() {
        let mut app = App::new();
        app.add_event::<ForgeryActionEvent>();
        app.add_systems(Update, process_mnestic_archiver_system);

        let pop = app.world_mut().spawn((
            Pop,
            Memories::default(),
            StressTracker { accumulated_stress: 80.0 },
            ForgedMemories::default(),
        )).id();

        app.world_mut().send_event(ForgeryActionEvent { pop });
        app.update();

        let stress = app.world().get::<StressTracker>(pop).unwrap();
        assert_eq!(stress.accumulated_stress, 0.0);
    }

    #[test]
    fn test_truth_outbreak_triggers_reality_collapse() {
        let mut app = App::new();
        app.add_event::<TruthOutbreakEvent>();
        app.add_systems(Update, process_truth_outbreak_system);

        let pop1 = app.world_mut().spawn((Pop, ForgedMemories { forged: true }, MentalState::Normal)).id();
        let pop2 = app.world_mut().spawn((Pop, ForgedMemories { forged: false }, MentalState::Normal)).id();

        app.world_mut().send_event(TruthOutbreakEvent);
        app.update();

        assert_eq!(*app.world().get::<MentalState>(pop1).unwrap(), MentalState::Broken(MentalBreakType::Vandalize));
        assert_eq!(*app.world().get::<MentalState>(pop2).unwrap(), MentalState::Normal);
    }
}
