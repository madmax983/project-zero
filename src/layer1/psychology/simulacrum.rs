use bevy::prelude::Transform;
use bevy_ecs::prelude::*;

use crate::layer1::entities::pop::Pop;
use crate::layer1::psychology::memory::{Memories, MemoryType};

/// Building component that broadcasts fake positive memories.
#[derive(Component)]
pub struct Simulacrum {
    /// Radius of effect.
    pub radius: f32,
    /// Is the building actively broadcasting.
    pub active: bool,
}

/// Broadcasts fake positive memories and removes negative memories for pops in radius.
pub fn simulacrum_broadcast_system(
    mut pops: Query<(&Transform, &mut Memories), With<Pop>>,
    simulacrums: Query<(&Transform, &Simulacrum)>,
) {
    for (sim_transform, simulacrum) in simulacrums.iter() {
        if !simulacrum.active {
            continue;
        }

        for (pop_transform, mut pop_memory) in pops.iter_mut() {
            if sim_transform
                .translation
                .distance(pop_transform.translation)
                <= simulacrum.radius
            {
                // Add fake positive if not already present
                if !pop_memory
                    .items
                    .iter()
                    .any(|e| e.memory_type == MemoryType::FakePositive)
                {
                    // For the sake of the minimal implementation, we use tick 0.
                    // A more advanced system would track actual ticks or use a resource.
                    pop_memory.add(MemoryType::FakePositive, 0);
                }

                // Remove negative memories
                pop_memory
                    .items
                    .retain(|e| e.memory_type.base_mood_impact() >= 0.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::psychology::memory::ActiveMemory;
    use bevy_app::App;
    use bevy_app::Update;

    #[test]
    fn test_simulacrum_generates_fake_memories() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, simulacrum_broadcast_system);

        let mut initial_memories = Memories::default();
        initial_memories.items.push(ActiveMemory {
            memory_type: MemoryType::StarvationTrauma,
            added_at: 0,
            intensity: 1.0,
            forged: false,
        });

        let pop = app
            .world_mut()
            .spawn((Pop, initial_memories, Transform::from_xyz(0.0, 0.0, 0.0)))
            .id();

        app.world_mut().spawn((
            Simulacrum {
                radius: 10.0,
                active: true,
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        // Act
        app.update();

        // Assert
        let pop_memory = app.world().get::<Memories>(pop).unwrap();
        assert!(pop_memory
            .items
            .iter()
            .any(|e| e.memory_type == MemoryType::FakePositive));
    }

    #[test]
    fn test_simulacrum_erases_negative_memories() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, simulacrum_broadcast_system);

        let mut initial_memories = Memories::default();
        initial_memories.items.push(ActiveMemory {
            memory_type: MemoryType::StarvationTrauma,
            added_at: 0,
            intensity: 1.0,
            forged: false,
        });

        let pop = app
            .world_mut()
            .spawn((Pop, initial_memories, Transform::from_xyz(0.0, 0.0, 0.0)))
            .id();

        app.world_mut().spawn((
            Simulacrum {
                radius: 10.0,
                active: true,
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        // Act
        app.update();
        app.update(); // Simulate enough time passing

        // Assert
        let pop_memory = app.world().get::<Memories>(pop).unwrap();
        assert!(!pop_memory
            .items
            .iter()
            .any(|e| e.memory_type == MemoryType::StarvationTrauma));
    }

    #[test]
    fn test_simulacrum_only_affects_pops_in_radius() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, simulacrum_broadcast_system);

        let mut initial_memories = Memories::default();
        initial_memories.items.push(ActiveMemory {
            memory_type: MemoryType::StarvationTrauma,
            added_at: 0,
            intensity: 1.0,
            forged: false,
        });

        let pop_far = app
            .world_mut()
            .spawn((Pop, initial_memories, Transform::from_xyz(100.0, 0.0, 0.0)))
            .id();

        app.world_mut().spawn((
            Simulacrum {
                radius: 10.0,
                active: true,
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        // Act
        app.update();

        // Assert
        let pop_memory = app.world().get::<Memories>(pop_far).unwrap();
        assert!(pop_memory
            .items
            .iter()
            .any(|e| e.memory_type == MemoryType::StarvationTrauma));
        assert!(!pop_memory
            .items
            .iter()
            .any(|e| e.memory_type == MemoryType::FakePositive));
    }
}
