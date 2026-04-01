use crate::layer1::memory::{ActiveMemory, Memories, MemoryType};
use crate::layer1::pop::Pop;
use bevy_ecs::prelude::*;

/// Megastructure that broadcasts propaganda to the planet.
#[derive(Component)]
pub struct PropagandaMonolith;

/// Emitted periodically when a Propaganda Monolith is active.
#[derive(Event, Debug, Clone)]
pub struct PropagandaBroadcastEvent {
    pub entity: Entity,
}

/// A system that gradually replaces negative memories with `GoldenAge`.
pub fn propaganda_broadcast_system(
    mut events: EventReader<PropagandaBroadcastEvent>,
    mut pop_query: Query<&mut Memories, With<Pop>>,
) {
    let broadcast_count = events.read().count();
    if broadcast_count == 0 {
        return;
    }

    for mut memories in pop_query.iter_mut() {
        for _ in 0..broadcast_count {
            // Find a negative memory
            let negative_idx = memories.items.iter().position(|m| {
                matches!(
                    m.memory_type,
                    MemoryType::WitnessedDeath
                        | MemoryType::StarvationTrauma
                        | MemoryType::LostLimb
                        | MemoryType::MascotDeath
                        | MemoryType::SleptInAwfulRoom
                        | MemoryType::AteInAwfulRoom
                )
            });

            if let Some(idx) = negative_idx {
                // Erase it and add GoldenAge
                memories.items.remove(idx);
                memories.items.push(ActiveMemory {
                    memory_type: MemoryType::GoldenAge,
                    added_at: 0,
                    intensity: 1.0,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::memory::{ActiveMemory, Memories};
    use crate::layer1::pop::Pop;

    #[test]
    fn test_monolith_erases_negative_memories() {
        let mut world = World::new();
        world.init_resource::<Events<PropagandaBroadcastEvent>>();

        let monolith = world.spawn(PropagandaMonolith).id();

        let pop = world
            .spawn((
                Pop,
                Memories {
                    items: vec![ActiveMemory {
                        memory_type: MemoryType::WitnessedDeath,
                        added_at: 0,
                        intensity: 1.0,
                    }],
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(propaganda_broadcast_system);

        let mut events = world.resource_mut::<Events<PropagandaBroadcastEvent>>();
        events.send(PropagandaBroadcastEvent { entity: monolith });

        schedule.run(&mut world);

        let memories = world.get::<Memories>(pop).unwrap();
        assert_eq!(memories.items.len(), 1);
        assert_eq!(memories.items[0].memory_type, MemoryType::GoldenAge);
    }
}
