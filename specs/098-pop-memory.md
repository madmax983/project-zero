# 098: Pop Memory

## Overview

Pops now have a **Memory** component. This allows them to "remember" significant events (Famines, Deaths, Celebrations) that occur during their lifetime. These memories provide a persistent history for each colonist and will later influence their mood and behavior (e.g., a "Survivor of the Long Hunger" might be anxious about food).

## Dependencies

- `003` — Pop Entity (Pop component)
- `010` — Chronicle System (Event structure)
- `031` — Morale System (Mood impact)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/memory_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::memory::{Memory, Memories, MemoryType, add_memory_system, MemoryEvent};
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_memories_component_defaults() {
        let memories = Memories::default();
        assert!(memories.items.is_empty());
    }

    #[test]
    fn test_add_memory_via_event() {
        let mut world = World::new();
        world.insert_resource(SimulationTime { tick: 100, ..Default::default() });

        let pop = world.spawn((Pop, Memories::default())).id();

        // Send a memory event
        world.send_event(MemoryEvent {
            target: pop,
            memory_type: MemoryType::WitnessedDeath,
            text: "Witnessed a death".to_string(),
            sentiment: -10.0,
        });

        // Run system to process event
        let mut schedule = Schedule::default();
        schedule.add_systems(add_memory_system);
        schedule.run(&mut world);

        let memories = world.get::<Memories>(pop).unwrap();
        assert_eq!(memories.items.len(), 1);
        assert_eq!(memories.items[0].text, "Witnessed a death");
        assert_eq!(memories.items[0].timestamp, 100);
        assert_eq!(memories.items[0].memory_type, MemoryType::WitnessedDeath);
    }

    #[test]
    fn test_memory_sentiment_impact() {
        // Future proofing: Ensure we can query total sentiment from memories
        let mut memories = Memories::default();
        memories.items.push(Memory {
            text: "Good times".into(),
            sentiment: 5.0,
            timestamp: 0,
            memory_type: MemoryType::General,
        });
        memories.items.push(Memory {
            text: "Bad times".into(),
            sentiment: -2.0,
            timestamp: 0,
            memory_type: MemoryType::General,
        });

        assert_eq!(memories.total_sentiment(), 3.0);
    }

    #[test]
    fn test_memory_capacity_limit() {
        let mut memories = Memories::default();
        for i in 0..50 {
            memories.add(Memory {
                text: format!("Mem {}", i),
                sentiment: 0.0,
                timestamp: i as u32,
                memory_type: MemoryType::General,
            });
        }

        // Assuming max capacity is e.g. 20
        // The implementation should drop oldest memories
        assert!(memories.items.len() <= 20);
        // Should keep the newest
        assert_eq!(memories.items.last().unwrap().text, "Mem 49");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Memory Structures

```rust
// src/layer1/memory.rs

use bevy_ecs::prelude::*;
use crate::shared::time::SimulationTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryType {
    General,
    WitnessedDeath,
    SurvivedFamine,
    Celebration,
}

#[derive(Debug, Clone)]
pub struct Memory {
    pub text: String,
    pub sentiment: f32,
    pub timestamp: u64, // Use SimulationTime tick
    pub memory_type: MemoryType,
}

#[derive(Component, Default, Debug)]
pub struct Memories {
    pub items: Vec<Memory>,
}

impl Memories {
    const MAX_MEMORIES: usize = 20;

    pub fn add(&mut self, memory: Memory) {
        self.items.push(memory);
        if self.items.len() > Self::MAX_MEMORIES {
            self.items.remove(0); // Remove oldest
        }
    }

    pub fn total_sentiment(&self) -> f32 {
        self.items.iter().map(|m| m.sentiment).sum()
    }
}

#[derive(Event)]
pub struct MemoryEvent {
    pub target: Entity,
    pub memory_type: MemoryType,
    pub text: String,
    pub sentiment: f32,
}
```

### 2. Implement System

```rust
// src/layer1/memory.rs

pub fn add_memory_system(
    mut events: EventReader<MemoryEvent>,
    mut query: Query<&mut Memories>,
    time: Res<SimulationTime>,
) {
    for event in events.read() {
        if let Ok(mut memories) = query.get_mut(event.target) {
            memories.add(Memory {
                text: event.text.clone(),
                sentiment: event.sentiment,
                timestamp: time.tick,
                memory_type: event.memory_type.clone(),
            });
        }
    }
}
```

### 3. Register

- Add `mod memory;` to `layer1/mod.rs`.
- Register `add_memory_system` in `main.rs` (or `layer1/mod.rs` plugin).
- Add `add_event::<MemoryEvent>()` to app.

## REFACTOR Phase: Quality & Design

- **Decay**: In the future, memory sentiment should decay towards 0 over time (healing/forgetting).
- **Integration**: Hook this into `pop_death_system` (when it's implemented) or `famine_system`.
- **UI**: Display memories in the Pop Inspector UI.

## Acceptance Criteria

- [ ] `Memories` component exists.
- [ ] `MemoryEvent` allows adding memories to specific pops.
- [ ] Memories are capped at a reasonable limit (FIFO).
- [ ] `total_sentiment` calculates the sum of memory impacts.
- [ ] Tests pass.

## Questions

- Should memories reference other Entities? (For MVP, text is enough).
- Should memories be saved/loaded? (Yes, they are components).
