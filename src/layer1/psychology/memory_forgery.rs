use bevy_ecs::prelude::*;

use crate::layer1::morale::Morale;
use crate::layer1::psychology::memory::{Memories, MemoryType};
use crate::layer1::psychology::stress::{Breakdown, BreakdownType, StressTracker};

#[derive(Component)]
pub struct MnesticArchiver;

#[derive(Component, Default)]
pub struct ForgedMemory;

#[derive(Event)]
pub struct EraseMemoryEvent {
    pub target: Entity,
}

#[derive(Event)]
pub struct TruthOutbreakEvent;


pub fn process_memory_forgery_system(
    mut commands: Commands,
    mut events: EventReader<EraseMemoryEvent>,
    mut pops: Query<(&mut Memories, &mut Morale, &mut StressTracker)>,
) {
    for event in events.read() {
        if let Ok((mut memories, mut morale, mut stress)) = pops.get_mut(event.target) {
            // Remove memories with negative base mood impact
            memories.items.retain(|m| m.memory_type.base_mood_impact() >= 0.0);

            // Insert fake positive memory
            if !memories.items.iter().any(|m| m.memory_type == MemoryType::FakePositive) {
                memories.add(MemoryType::FakePositive, 0); // Using 0 as a placeholder tick for MVP
            }

            // Mark as forged
            commands.entity(event.target).insert(ForgedMemory);

            // Max out morale and reset stress
            morale.value = 100.0;
            stress.accumulated_stress = 0.0;
        }
    }
}

pub fn process_truth_outbreak_system(
    mut commands: Commands,
    mut events: EventReader<TruthOutbreakEvent>,
    pops: Query<Entity, With<ForgedMemory>>,
) {
    for _event in events.read() {
        for entity in pops.iter() {
            commands.entity(entity).insert(Breakdown {
                breakdown_type: BreakdownType::RealityCollapse,
                duration_remaining: 1000,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use crate::layer1::pop::Pop;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<EraseMemoryEvent>();
        app.add_event::<TruthOutbreakEvent>();
        app.add_systems(bevy_app::Update, (process_memory_forgery_system, process_truth_outbreak_system));
        app
    }

    #[test]
    fn test_mnestic_archiver_erases_bad_memories() {
        let mut app = setup_app();

        let mut memories = Memories::default();
        memories.add(MemoryType::StarvationTrauma, 0); // Negative memory

        let pop = app.world_mut().spawn((
            Pop,
            memories,
            Morale { value: 50.0, modifiers: vec![] },
            StressTracker { accumulated_stress: 50.0 },
        )).id();

        app.world_mut().resource_mut::<Events<EraseMemoryEvent>>().send(EraseMemoryEvent { target: pop });

        app.update();

        let pop_memory = app.world().get::<Memories>(pop).unwrap();

        // StarvationTrauma should be removed
        assert!(!pop_memory.items.iter().any(|m| m.memory_type == MemoryType::StarvationTrauma));
        // FakePositive should be added
        assert!(pop_memory.items.iter().any(|m| m.memory_type == MemoryType::FakePositive));
        // Should have ForgedMemory component
        assert!(app.world().get::<ForgedMemory>(pop).is_some());
    }

    #[test]
    fn test_mnestic_archiver_boosts_morale() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            Pop,
            Memories::default(),
            Morale { value: 10.0, modifiers: vec![] },
            StressTracker { accumulated_stress: 50.0 },
        )).id();

        app.world_mut().resource_mut::<Events<EraseMemoryEvent>>().send(EraseMemoryEvent { target: pop });

        app.update();

        let morale = app.world().get::<Morale>(pop).unwrap();
        let stress = app.world().get::<StressTracker>(pop).unwrap();

        assert_eq!(morale.value, 100.0);
        assert_eq!(stress.accumulated_stress, 0.0);
    }

    #[test]
    fn test_truth_outbreak_triggers_reality_collapse() {
        let mut app = setup_app();

        let pop = app.world_mut().spawn((
            Pop,
            ForgedMemory,
        )).id();

        app.world_mut().resource_mut::<Events<TruthOutbreakEvent>>().send(TruthOutbreakEvent);

        app.update();

        let breakdown = app.world().get::<Breakdown>(pop).unwrap();
        assert_eq!(breakdown.breakdown_type, BreakdownType::RealityCollapse);
        assert_eq!(breakdown.duration_remaining, 1000);
    }
}
