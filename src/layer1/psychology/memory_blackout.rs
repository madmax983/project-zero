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
