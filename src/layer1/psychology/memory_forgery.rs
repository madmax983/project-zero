use crate::layer1::psychology::memory::{Memories, MemoryType};
use crate::layer1::social::unrest::{MentalBreakType, MentalState};
use crate::layer1::stress::StressTracker;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct MnesticArchiver;

#[derive(Event, Default)]
pub struct TruthOutbreakEvent;

pub fn mnestic_archiver_system(
    mut query: Query<(&mut Memories, &mut StressTracker), With<MnesticArchiver>>,
) {
    for (mut memories, mut stress) in query.iter_mut() {
        let mut has_bad_memory = false;

        memories.items.retain(|m| {
            if m.memory_type.base_mood_impact() < 0.0 {
                has_bad_memory = true;
                false
            } else {
                true
            }
        });

        if has_bad_memory {
            memories.add_forged(MemoryType::AteFineMeal, 0);
            stress.accumulated_stress = 0.0;
        }
    }
}

pub fn truth_outbreak_system(
    mut events: EventReader<TruthOutbreakEvent>,
    mut query: Query<(&Memories, &mut MentalState)>,
) {
    for _ in events.read() {
        for (memories, mut mental_state) in query.iter_mut() {
            if memories.items.iter().any(|m| m.forged) {
                *mental_state = MentalState::Broken(MentalBreakType::RealityCollapse);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_mnestic_archiver_erases_bad_memories() {
        let mut app = bevy_app::App::new();
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(mnestic_archiver_system);

        let entity = app
            .world_mut()
            .spawn((
                Pop,
                Memories::default(),
                StressTracker {
                    accumulated_stress: 50.0,
                },
                MnesticArchiver,
            ))
            .id();
        app.world_mut()
            .get_mut::<Memories>(entity)
            .unwrap()
            .add(MemoryType::StarvationTrauma, 0);

        schedule.run(app.world_mut());

        let memories = app.world().get::<Memories>(entity).unwrap();
        assert!(!memories
            .items
            .iter()
            .any(|m| m.memory_type == MemoryType::StarvationTrauma));
        assert!(memories
            .items
            .iter()
            .any(|m| m.memory_type == MemoryType::AteFineMeal && m.forged));
    }

    #[test]
    fn test_mnestic_archiver_boosts_morale() {
        let mut app = bevy_app::App::new();
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(mnestic_archiver_system);

        let entity = app
            .world_mut()
            .spawn((
                Pop,
                Memories::default(),
                StressTracker {
                    accumulated_stress: 50.0,
                },
                MnesticArchiver,
            ))
            .id();
        app.world_mut()
            .get_mut::<Memories>(entity)
            .unwrap()
            .add(MemoryType::StarvationTrauma, 0);

        schedule.run(app.world_mut());

        let stress = app.world().get::<StressTracker>(entity).unwrap();
        assert!((stress.accumulated_stress - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_truth_outbreak_triggers_reality_collapse() {
        let mut app = bevy_app::App::new();
        app.world_mut()
            .init_resource::<Events<TruthOutbreakEvent>>();
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(truth_outbreak_system);

        let entity = app
            .world_mut()
            .spawn((Pop, Memories::default(), MentalState::Normal))
            .id();
        app.world_mut()
            .get_mut::<Memories>(entity)
            .unwrap()
            .add_forged(MemoryType::AteFineMeal, 0);

        app.world_mut().send_event(TruthOutbreakEvent);

        schedule.run(app.world_mut());

        let mental_state = app.world().get::<MentalState>(entity).unwrap();
        assert_eq!(
            *mental_state,
            MentalState::Broken(MentalBreakType::RealityCollapse)
        );
    }
}
