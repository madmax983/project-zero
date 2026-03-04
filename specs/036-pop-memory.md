# 036: Pop Memory

## Overview

Pops accumulate persistent memories from significant events (Witnessing Death, Starvation, combat, great meals). Unlike transient `Thoughts` (flavor text), `Memories` have mechanical weight: they modify Morale over time, decaying slowly. This adds psychological depth and consequences to colony management.

## Dependencies

- `005` — Pop Needs (Hunger, Rest)
- `031` — Pop Morale (Base morale calculation)
- `010` — Chronicle System (Event definitions/IDs)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/memory_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::needs::Needs;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_add_memory() {
        let mut memories = Memories::default();
        memories.add(MemoryType::WitnessedDeath, 100);

        assert_eq!(memories.items.len(), 1);
        assert_eq!(memories.items[0].memory_type, MemoryType::WitnessedDeath);
        assert_eq!(memories.items[0].added_at, 100);
        assert!((memories.items[0].intensity - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_memory_decay() {
        let mut memories = Memories::default();
        memories.add(MemoryType::WitnessedDeath, 0);

        // Decay logic: Intensity reduces by decay_rate * ticks
        // Assume WitnessedDeath decay_rate = 0.001 per tick
        memories.decay(100);

        assert!(memories.items[0].intensity < 1.0);
        assert!(memories.items[0].intensity > 0.0);
    }

    #[test]
    fn test_memory_removal_when_faded() {
        let mut memories = Memories::default();
        memories.add(MemoryType::WitnessedDeath, 0);

        // Force decay to zero
        memories.items[0].intensity = 0.0;
        memories.decay(1);

        assert!(memories.items.is_empty());
    }

    #[test]
    fn test_calculate_effective_morale() {
        let needs = Needs { hunger: 0.5, rest: 0.5, leisure: 0.5 };
        let mut memories = Memories::default();

        // Base morale = (0.5+0.5+0.5)/3 = 0.5
        let base = needs.morale();

        // WitnessedDeath: -0.2 mood impact at max intensity
        memories.add(MemoryType::WitnessedDeath, 0);

        let effective = calculate_effective_morale(&needs, &memories);

        // 0.5 - 0.2 = 0.3
        assert!((effective - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_multiple_memories_stack() {
        let needs = Needs { hunger: 0.5, rest: 0.5, leisure: 0.5 };
        let mut memories = Memories::default();

        memories.add(MemoryType::WitnessedDeath, 0); // -0.2
        memories.add(MemoryType::AteFineMeal, 0);    // +0.1 (hypothetical)

        let effective = calculate_effective_morale(&needs, &memories);

        // 0.5 - 0.2 + 0.1 = 0.4
        assert!((effective - 0.4).abs() < 0.001);
    }

    #[test]
    fn test_morale_clamping() {
        let needs = Needs { hunger: 1.0, rest: 1.0, leisure: 1.0 }; // Base 1.0
        let mut memories = Memories::default();
        memories.add(MemoryType::AteFineMeal, 0); // +0.1

        let effective = calculate_effective_morale(&needs, &memories);
        assert!(effective <= 1.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Memory Types and Structs

```rust
// src/layer1/memory.rs

use bevy_ecs::prelude::*;
use crate::layer1::needs::Needs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryType {
    WitnessedDeath,
    StarvationTrauma,
    AteFineMeal,
    WonFight,
}

impl MemoryType {
    pub fn base_mood_impact(&self) -> f32 {
        match self {
            MemoryType::WitnessedDeath => -0.2,
            MemoryType::StarvationTrauma => -0.15,
            MemoryType::AteFineMeal => 0.1,
            MemoryType::WonFight => 0.05,
        }
    }

    pub fn decay_rate(&self) -> f32 {
        // Ticks to fade completely
        match self {
            MemoryType::WitnessedDeath => 0.0005,    // Slow fade (2000 ticks)
            MemoryType::StarvationTrauma => 0.001,   // Medium
            MemoryType::AteFineMeal => 0.002,        // Fast (500 ticks)
            MemoryType::WonFight => 0.002,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActiveMemory {
    pub memory_type: MemoryType,
    pub added_at: u64,
    pub intensity: f32, // 0.0 to 1.0
}

#[derive(Component, Default, Debug, Clone)]
pub struct Memories {
    pub items: Vec<ActiveMemory>,
}

impl Memories {
    pub fn add(&mut self, memory_type: MemoryType, current_tick: u64) {
        self.items.push(ActiveMemory {
            memory_type,
            added_at: current_tick,
            intensity: 1.0,
        });
    }

    pub fn decay(&mut self, ticks_passed: u64) {
        let ticks_f32 = ticks_passed as f32;
        for memory in &mut self.items {
            memory.intensity -= memory.memory_type.decay_rate() * ticks_f32;
        }
        self.items.retain(|m| m.intensity > 0.0);
    }
}
```

### 2. Morale Calculation Helper

```rust
// src/layer1/memory.rs

pub fn calculate_effective_morale(needs: &Needs, memories: &Memories) -> f32 {
    let base = needs.morale();
    let memory_modifier: f32 = memories.items.iter()
        .map(|m| m.memory_type.base_mood_impact() * m.intensity)
        .sum();

    (base + memory_modifier).clamp(0.0, 1.0)
}
```

### 3. Memory Decay System

```rust
// src/layer1/memory.rs

use crate::shared::time::SimulationTime;

pub fn memory_decay_system(
    mut query: Query<&mut Memories>,
    // We assume this runs every tick, or we can use time delta if needed
) {
    for mut memories in &mut query {
        memories.decay(1);
    }
}
```

## REFACTOR Phase: Quality & Design

- **Integration**: Update `spawn_initial_pops` in `src/layer1/pop.rs` to include `Memories::default()`.
- **Usage**: Update usages of `needs.morale()` to `calculate_effective_morale(&needs, &memories)` where applicable (e.g. `work_execution_system`).
- **Optimization**: `Memories` `Vec` is likely small (<10), so linear scan is fine.
- **Inspector**: Expose memories in the UI inspector (Spec 015).

## Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `Memories` component exists and can be added to entities.
- [ ] Memories decay over time and are removed when intensity <= 0.
- [ ] `calculate_effective_morale` correctly modifies base morale.
- [ ] `cargo test` passes.
- [ ] `cargo clippy` passes.

## Technical Guidance

- Ensure `needs.morale()` is available (from Spec 031). If Spec 031 is not fully merged, define a local fallback or mock for the test.
- The `decay` method takes `ticks_passed` to allow for "catch-up" logic if systems don't run every tick, though `memory_decay_system` runs per tick usually.

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
