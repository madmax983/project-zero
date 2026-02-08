# 061: Cultural Artifacts (Art)

## Overview

Statues and specific buildings are not just generic objects; they are cultural artifacts that capture the history of the colony. When constructed, they gain an `Art` component depicting a significant event from the `Chronicle` or a collective memory. Observing these artifacts provides a mood boost to Pops, reinforcing their connection to the colony's history.

## Dependencies

- `044` — Horticulture and Beauty (Defines `Statue` building)
- `010` — Chronicle System (Source of history)
- `036` — Pop Memory (Source of mood/thoughts)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/art_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::chronicle::{Chronicle, EventImportance};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::memory::{Memories, MemoryType};
    use bevy_ecs::prelude::*;

    // 1. Art Creation
    #[test]
    fn test_statue_gains_art_component() {
        let mut world = World::new();
        // Setup dependencies
        world.insert_resource(Chronicle::default());
        // Initialize Art plugin/systems if needed, or run manually

        // Spawn a Statue
        let statue = world.spawn((
            Building { building_type: BuildingType::Statue },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Run system that assigns art
        // (Assuming `art_generation_system` runs on Added<Building>)
        crate::layer1::art::art_generation_system(&mut world);

        // Assert Art component exists
        let art = world.get::<crate::layer1::art::Art>(statue);
        assert!(art.is_some(), "Statue should have Art component");
    }

    #[test]
    fn test_art_captures_chronicle_event() {
        let mut world = World::new();
        let mut chronicle = Chronicle::default();
        chronicle.add_event(0, "Colony Founded".to_string(), EventImportance::Legendary);
        world.insert_resource(chronicle);

        let statue = world.spawn((
            Building { building_type: BuildingType::Statue },
            GridPosition { x: 5, y: 5 },
        )).id();

        crate::layer1::art::art_generation_system(&mut world);

        let art = world.get::<crate::layer1::art::Art>(statue).unwrap();
        assert!(!art.description.is_empty());
        assert!(art.description.contains("Colony Founded"));
    }

    #[test]
    fn test_art_fallback_if_chronicle_empty() {
        let mut world = World::new();
        world.insert_resource(Chronicle::default()); // Empty

        let statue = world.spawn((
            Building { building_type: BuildingType::Statue },
            GridPosition { x: 5, y: 5 },
        )).id();

        crate::layer1::art::art_generation_system(&mut world);

        let art = world.get::<crate::layer1::art::Art>(statue).unwrap();
        assert_eq!(art.description, "Abstract Art");
    }

    // 2. Art Observation
    #[test]
    fn test_observing_art_gives_memory_buff() {
        let mut world = World::new();

        // Spawn Art
        world.spawn((
            Building { building_type: BuildingType::Statue },
            GridPosition { x: 5, y: 5 },
            crate::layer1::art::Art { description: "Great Art".to_string() },
        ));

        // Spawn Pop nearby
        let pop = world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 }, // Same tile
            Memories::default(),
        )).id();

        // Run observation system
        crate::layer1::art::art_observation_system(&mut world);

        // Check Pop memories
        let memories = world.get::<Memories>(pop).unwrap();
        // Assume we add a specific "AdmiredArt" memory type
        let found = memories.items.iter().any(|m| m.memory_type == MemoryType::AdmiredArt);
        assert!(found, "Pop should have AdmiredArt memory");
    }

    #[test]
    fn test_observing_art_distance_limit() {
        let mut world = World::new();

        // Spawn Art
        world.spawn((
            Building { building_type: BuildingType::Statue },
            GridPosition { x: 5, y: 5 },
            crate::layer1::art::Art { description: "Great Art".to_string() },
        ));

        // Spawn Pop far away
        let pop = world.spawn((
            Pop,
            GridPosition { x: 10, y: 10 },
            Memories::default(),
        )).id();

        crate::layer1::art::art_observation_system(&mut world);

        let memories = world.get::<Memories>(pop).unwrap();
        let found = memories.items.iter().any(|m| m.memory_type == MemoryType::AdmiredArt);
        assert!(!found, "Pop too far away should not admire art");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. `Art` Component

```rust
// src/layer1/art.rs
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct Art {
    pub description: String,
}
```

### 2. Update `MemoryType`

Add `AdmiredArt` to `MemoryType` enum in `src/layer1/memory.rs`.

```rust
pub enum MemoryType {
    // ... existing ...
    AdmiredArt, // New type
}

// Update base_mood_impact -> +0.05
// Update decay_rate -> Fast (e.g., 0.01)
```

### 3. Art Generation System

```rust
// src/layer1/art.rs
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::chronicle::Chronicle;
use rand::Rng;

pub fn art_generation_system(
    mut commands: Commands,
    // Use Added filter to only process new buildings
    query: Query<(Entity, &Building), Added<Building>>,
    chronicle: Res<Chronicle>,
) {
    let mut rng = rand::thread_rng();

    for (entity, building) in &query {
        if building.building_type == BuildingType::Statue {
            let description = if chronicle.events.is_empty() {
                "Abstract Art".to_string()
            } else {
                // Pick random event
                let idx = rng.gen_range(0..chronicle.events.len());
                let event = &chronicle.events[idx];
                format!("Art depicting: {}", event.text)
            };

            commands.entity(entity).insert(Art { description });
        }
    }
}
```

### 4. Art Observation System

```rust
// src/layer1/art.rs
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::memory::{Memories, MemoryType};
use crate::shared::time::SimulationTime;

pub fn art_observation_system(
    art_query: Query<(&GridPosition, &Art)>,
    mut pop_query: Query<(&GridPosition, &mut Memories), With<Pop>>,
    time: Res<SimulationTime>,
) {
    for (pop_pos, mut memories) in &mut pop_query {
        // Optimization: Could use spatial index, but brute force O(N*M) is fine for MVP (few statues)
        for (art_pos, _) in &art_query {
            // Check distance (Chebyshev or Manhattan)
            let dx = (pop_pos.x - art_pos.x).abs();
            let dy = (pop_pos.y - art_pos.y).abs();

            // Range 2 tiles
            if dx <= 2 && dy <= 2 {
                // Check if already has memory to avoid spamming
                let has_memory = memories.items.iter().any(|m| m.memory_type == MemoryType::AdmiredArt);
                if !has_memory {
                    memories.add(MemoryType::AdmiredArt, time.tick);
                    // Only admire one piece per tick
                    break;
                }
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Quality Levels**: Use `Crafter` skill to determine Art quality (impacting mood bonus).
- **Commissioning**: Allow player to specify the subject via UI.
- **Thought Cooldown**: Prevent `AdmiredArt` from triggering too frequently (add a separate cooldown timer or rely on memory duration).
- **Optimization**: Use spatial hashing for observation if Art count grows large.

## Acceptance Criteria

- [ ] `Art` component exists.
- [ ] `MemoryType::AdmiredArt` exists with positive mood impact.
- [ ] Statues automatically gain `Art` upon construction.
- [ ] Art descriptions reference valid `Chronicle` events.
- [ ] Pops near Art gain `AdmiredArt` memory.
- [ ] Tests pass.

## Questions

*Builder: Should Art also emit Beauty?*
Answer: The `Statue` building already emits Beauty via Spec `044`. This `Art` component adds the specific historical flavor and mood memory on top.
