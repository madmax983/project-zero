# The Propaganda Simulacrum

## 1. Overview
**Layer:** 1 -> Cross-layer
**Fantasy:** When the truth is too bleak, you build a machine to tell a better lie, but eventually, people start believing the lie more than reality.
**Mechanic:** A high-tech `Simulacrum` building generates a continuous, artificial stream of fake "Good News" and "Heroic Memories" that replace genuine, negative memories in nearby Pops. It provides a massive, artificial Morale boost but slowly replaces the colony's actual history with a fabricated, perfectly optimistic narrative.

## 2. Dependencies
- `031-pop-morale.md`
- `036-pop-memory.md`

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_simulacrum_generates_fake_memories() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, simulacrum_broadcast_system);

        let pop = app.world.spawn((
            Pop,
            Memory {
                events: vec![MemoryEvent::new("Starvation", MemoryType::Negative)],
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let simulacrum = app.world.spawn((
            Simulacrum {
                radius: 10.0,
                active: true,
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        // Act
        app.update();

        // Assert
        let pop_memory = app.world.get::<Memory>(pop).unwrap();
        assert!(pop_memory.events.iter().any(|e| e.event_type == MemoryType::FakePositive));
    }

    #[test]
    fn test_simulacrum_erases_negative_memories() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, simulacrum_broadcast_system);

        let pop = app.world.spawn((
            Pop,
            Memory {
                events: vec![MemoryEvent::new("Starvation", MemoryType::Negative)],
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        app.world.spawn((
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
        let pop_memory = app.world.get::<Memory>(pop).unwrap();
        assert!(!pop_memory.events.iter().any(|e| e.description == "Starvation"));
    }

    #[test]
    fn test_simulacrum_only_affects_pops_in_radius() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, simulacrum_broadcast_system);

        let pop_far = app.world.spawn((
            Pop,
            Memory {
                events: vec![MemoryEvent::new("Starvation", MemoryType::Negative)],
            },
            Transform::from_xyz(100.0, 0.0, 0.0),
        )).id();

        app.world.spawn((
            Simulacrum {
                radius: 10.0,
                active: true,
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        // Act
        app.update();

        // Assert
        let pop_memory = app.world.get::<Memory>(pop_far).unwrap();
        assert!(pop_memory.events.iter().any(|e| e.description == "Starvation"));
        assert!(!pop_memory.events.iter().any(|e| e.event_type == MemoryType::FakePositive));
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(PartialEq, Clone)]
pub enum MemoryType {
    Positive,
    Negative,
    FakePositive,
}

#[derive(Clone)]
pub struct MemoryEvent {
    pub description: String,
    pub event_type: MemoryType,
}

impl MemoryEvent {
    pub fn new(desc: &str, evt_type: MemoryType) -> Self {
        Self {
            description: desc.to_string(),
            event_type: evt_type,
        }
    }
}

#[derive(Component)]
pub struct Memory {
    pub events: Vec<MemoryEvent>,
}

#[derive(Component)]
pub struct Simulacrum {
    pub radius: f32,
    pub active: bool,
}

pub fn simulacrum_broadcast_system(
    mut pops: Query<(&Transform, &mut Memory), With<Pop>>,
    simulacrums: Query<(&Transform, &Simulacrum)>,
) {
    for (sim_transform, simulacrum) in simulacrums.iter() {
        if !simulacrum.active {
            continue;
        }

        for (pop_transform, mut pop_memory) in pops.iter_mut() {
            if sim_transform.translation.distance(pop_transform.translation) <= simulacrum.radius {
                // Add fake positive
                if !pop_memory.events.iter().any(|e| e.event_type == MemoryType::FakePositive) {
                    pop_memory.events.push(MemoryEvent::new("The Golden Age", MemoryType::FakePositive));
                }

                // Remove negative memories
                pop_memory.events.retain(|e| e.event_type != MemoryType::Negative);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Move `MemoryType` and `MemoryEvent` to a shared `memory` module if they don't exist yet, or integrate with existing ones.
- Refactor the replacement logic to happen over time instead of instantly in one tick (e.g., using a timer or accumulator per pop).
- Performance: Instead of nested O(N*M) loop, we should use spatial partitioning or spatial queries if there are many simulacrums and pops.
- The `FakePositive` memory should dynamically fetch strings from a lore generator or data-driven file.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Simulacrum successfully replaces negative memories with fake positives in affected pops.

## 7. Technical Guidance
- The `simulacrum_broadcast_system` needs to run after the system that generates negative memories.
- Consider adding a `Deluded` trait to pops whose memories are majority fake to influence other behaviors (like ignoring threats).

## 8. Questions
*Builder: add questions here if spec is unclear.*
