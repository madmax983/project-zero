# Spec 341: Pop Memory

## 1. Overview
This feature introduces "Memories" to Pops. Pops will accumulate memory tokens when they witness significant events (like famines, surviving raids, arriving on a colony ship). These memories influence their behavior and mood over time. By sharing common experiences, a "Colony Identity" can organically emerge, affecting traits like resilience.

## 2. Dependencies
- `Needs` and `StressTracker` (for mood impact)
- `ChronicleEvent` or equivalent event systems to capture when something notable happens to the colony or an individual pop.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::pre.lude::*;

    #[test]
    fn test_pop_gains_memory_from_event() {
        // Arrange
        let mut app = App::new();
        app.add_event::<WitnessEvent>();
        app.add_systems(Update, record_pop_memory_system);

        let pop = app.world_mut().spawn(PopMemoryTracker::default()).id();

        // Act
        app.world_mut().send_event(WitnessEvent {
            pop_entity: pop,
            memory_type: MemoryType::SurvivedFamine,
            severity: 1.0,
        });
        app.update();

        // Assert
        let memory_tracker = app.world().get::<PopMemoryTracker>(pop).unwrap();
        assert!(memory_tracker.has_memory(MemoryType::SurvivedFamine));
    }

    #[test]
    fn test_memory_influences_mood() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_memory_mood_modifiers);

        let pop = app.world_mut().spawn((
            PopMemoryTracker {
                memories: vec![Memory { memory_type: MemoryType::SurvivedFamine, weight: 1.0 }],
            },
            StressTracker::default(),
        )).id();

        // Act
        app.update();

        // Assert
        let stress = app.world().get::<StressTracker>(pop).unwrap();
        // A pop who survived a famine might have a persistent low-level stress modifier
        // or a resilience modifier. Test verifies the modifier was applied.
        assert!(stress.get_modifier(ModifierSource::Memory) != 0.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Event)]
pub struct WitnessEvent {
    pub pop_entity: Entity,
    pub memory_type: MemoryType,
    pub severity: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MemoryType {
    SurvivedFamine,
    ColonyFounder,
}

#[derive(Clone)]
pub struct Memory {
    pub memory_type: MemoryType,
    pub weight: f32,
}

#[derive(Component, Default)]
pub struct PopMemoryTracker {
    pub memories: Vec<Memory>,
}

impl PopMemoryTracker {
    pub fn has_memory(&self, mem_type: MemoryType) -> bool {
        self.memories.iter().any(|m| m.memory_type == mem_type)
    }
}

pub fn record_pop_memory_system(
    mut events: EventReader<WitnessEvent>,
    mut query: Query<&mut PopMemoryTracker>,
) {
    for event in events.read() {
        if let Ok(mut tracker) = query.get_mut(event.pop_entity) {
            tracker.memories.push(Memory {
                memory_type: event.memory_type,
                weight: event.severity,
            });
        }
    }
}

pub fn apply_memory_mood_modifiers(
    mut query: Query<(&PopMemoryTracker, &mut StressTracker)>,
) {
    // For now, minimal implementation to satisfy the test
    for (tracker, mut stress) in query.iter_mut() {
        if tracker.has_memory(MemoryType::SurvivedFamine) {
            stress.add_modifier(ModifierSource::Memory, 5.0); // Example modifier
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Memory Decay**: Memories should probably fade over time (weight decreases) unless reinforced.
- **Data Structure**: Consider using a `HashMap<MemoryType, f32>` instead of a `Vec<Memory>` for faster lookups and easier accumulation of weights for the same memory type.
- **Cultural Shared Memories**: If >50% of the colony has a specific memory, it could become a global trait.
- **Integration**: Tie this into the existing `Chronicle` system so memories are generated automatically when major chronicle events happen.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `pop_memory.rs`.
- [ ] Event listener successfully attaches `Memory` records to `PopMemoryTracker`.

## 7. Technical Guidance
- **Layer**: Layer 1 (Colony/Pop scale).
- Place the core logic in `src/layer1/pop/memory.rs` or similar.
- Use `events.read()` for Bevy 0.13+ rather than `iter()`.
- Be mindful of memory footprint; thousands of pops each with dozens of memories could become a performance concern. Limit the number of active memories per pop, prioritizing the most heavily weighted ones.

## 8. Questions
*Builder: add questions here if spec is unclear.*
