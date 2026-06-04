use crate::layer1::entities::pop::Pop;
use crate::layer1::psychology::memory::Memories;
use crate::layer1::skills::Skills;
use crate::layer1::social::Relationships;
use bevy::prelude::*;

#[derive(Event)]
pub struct MemoryBlackoutEvent {
    pub start_time: u64,
    pub end_time: u64,
}

#[allow(clippy::type_complexity)]
pub fn process_memory_blackout(
    mut events: EventReader<MemoryBlackoutEvent>,
    mut query: Query<
        (
            Option<&mut Memories>,
            Option<&mut Relationships>,
            Option<&mut Skills>,
        ),
        With<Pop>,
    >,
) {
    for event in events.read() {
        for (memory_opt, relations_opt, skills_opt) in query.iter_mut() {
            if let Some(mut memory) = memory_opt {
                memory
                    .items
                    .retain(|e| e.added_at < event.start_time || e.added_at > event.end_time);
            }
            if let Some(mut relations) = relations_opt {
                relations.affinities.clear();
            }
            if let Some(mut skills) = skills_opt {
                skills.xp.clear();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::psychology::memory::{Memories, MemoryType};
    use crate::layer1::skills::{SkillType, Skills};
    use crate::layer1::social::Relationships;

    #[test]
    fn should_process_memory_blackout_and_clear_data() {
        let mut app = App::new();
        app.add_event::<MemoryBlackoutEvent>();
        app.add_systems(Update, process_memory_blackout);

        // Pop with all components
        let mut memories = Memories { items: vec![] };
        memories.add(MemoryType::AteFineMeal, 10);
        memories.add(MemoryType::SawCorpse, 50);
        memories.add(MemoryType::WonFight, 90);

        let mut skills = Skills::default();
        skills.xp.insert(SkillType::Mining, 100.0);

        let other_entity = app.world_mut().spawn_empty().id();
        let mut relations = Relationships::default();
        relations.set_affinity(other_entity, 50.0);

        let pop_entity = app
            .world_mut()
            .spawn((Pop, memories, skills, relations))
            .id();

        // Pop without components (should not panic)
        let _empty_pop = app.world_mut().spawn(Pop).id();

        app.world_mut().send_event(MemoryBlackoutEvent {
            start_time: 20,
            end_time: 80,
        });

        app.update();

        let mem = app.world().get::<Memories>(pop_entity).unwrap();
        assert_eq!(mem.items.len(), 2);
        assert_eq!(mem.items[0].memory_type, MemoryType::AteFineMeal);
        assert_eq!(mem.items[1].memory_type, MemoryType::WonFight);

        let rel = app.world().get::<Relationships>(pop_entity).unwrap();
        assert!(rel.affinities.is_empty());

        let sk = app.world().get::<Skills>(pop_entity).unwrap();
        assert!(sk.xp.is_empty());
    }
}
