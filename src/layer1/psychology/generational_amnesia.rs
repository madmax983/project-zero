use crate::layer1::psychology::memory::Memories;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct GenerationalAmnesia {
    pub decay_rate: f32,
    pub current_amnesia: f32,
}

pub fn apply_amnesia_system(mut query: Query<&mut GenerationalAmnesia>, time: Res<SimulationTime>) {
    if time.speed == crate::shared::time::SimSpeed::Paused {
        return;
    }
    for mut amnesia in query.iter_mut() {
        amnesia.current_amnesia += amnesia.decay_rate;
    }
}

pub fn apply_amnesia_to_memories_system(
    mut query: Query<(&GenerationalAmnesia, &mut Memories)>,
    time: Res<SimulationTime>,
) {
    if time.speed == crate::shared::time::SimSpeed::Paused {
        return;
    }
    for (amnesia, mut memories) in query.iter_mut() {
        if amnesia.current_amnesia > 0.0 {
            // Accelerate memory decay based on amnesia level
            for memory in &mut memories.items {
                let decay = memory.memory_type.decay_rate() * amnesia.current_amnesia;
                memory.intensity = (memory.intensity - decay).max(0.0);
            }
            memories.items.retain(|m| m.intensity > 0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::entities::pop::Pop;
    use crate::layer1::psychology::memory::MemoryType;
    use bevy::prelude::*;

    #[test]
    fn test_amnesia_accumulation() {
        let mut app = App::new();

        app.insert_resource(SimulationTime {
            tick: 0,
            speed: crate::shared::time::SimSpeed::Normal,
        });

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                GenerationalAmnesia {
                    decay_rate: 0.1,
                    current_amnesia: 0.0,
                },
            ))
            .id();

        app.add_systems(Update, apply_amnesia_system);
        app.update();

        let amnesia = app.world().get::<GenerationalAmnesia>(pop_entity).unwrap();
        assert!((amnesia.current_amnesia - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_amnesia_affects_memories() {
        let mut app = App::new();

        app.insert_resource(SimulationTime {
            tick: 0,
            speed: crate::shared::time::SimSpeed::Normal,
        });

        let mut memories = Memories::default();
        memories.add(MemoryType::WitnessedDeath, 0);

        let pop_entity = app
            .world_mut()
            .spawn((
                Pop,
                GenerationalAmnesia {
                    decay_rate: 0.1,
                    current_amnesia: 10.0,
                },
                memories,
            ))
            .id();

        app.add_systems(Update, apply_amnesia_to_memories_system);
        app.update();

        let memories_after = app.world().get::<Memories>(pop_entity).unwrap();

        let initial_intensity = 1.0;
        let expected_decay = MemoryType::WitnessedDeath.decay_rate() * 10.0;
        let expected_intensity = (initial_intensity - expected_decay).max(0.0);

        assert!((memories_after.items[0].intensity - expected_intensity).abs() < f32::EPSILON);
    }
}
