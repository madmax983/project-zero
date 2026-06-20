# 692: Procedural Dialects

## Overview

A feature where the game logs and chatter generate slang based on colony events. Pops form their own "Dialect" through shared experiences (e.g., if a major fire kills many pops, "Fire" becomes a curse word; a legendary miner might have their name become synonymous with luck). This creates an emergent culture and sense of history within the colony's textual output.

## Dependencies

- `010` — Chronicle System
- `036` — Pop Memory

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::core::chronicle::{ChronicleEvent, EventImportance};
    use crate::layer1::entities::pop::Pop;
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_event_creates_dialect_slang() {
        let mut app = App::new();
        app.init_resource::<DialectManager>();

        // Trigger a major historical event
        app.world_mut().send_event(ChronicleEvent {
            tick: 1000,
            year: 1,
            text: "The Great Fire destroyed half the colony.".to_string(),
            importance: EventImportance::Major,
        });

        app.add_systems(Update, process_chronicle_events_for_dialect);
        app.update();

        let dialect = app.world().resource::<DialectManager>();
        // High severity fire event should create slang related to 'Fire'
        assert!(dialect.slang_dictionary.contains_key("fire"));
        assert_eq!(dialect.slang_dictionary.get("fire").unwrap().usage, SlangUsage::Curse);
    }

    #[test]
    fn test_new_pop_adopts_colony_dialect() {
        let mut app = App::new();
        app.init_resource::<DialectManager>();

        let mut dialect = app.world_mut().resource_mut::<DialectManager>();
        dialect.slang_dictionary.insert(
            "rust".to_string(),
            SlangEntry { usage: SlangUsage::Praise, weight: 1.0 }
        );

        // Spawn a new pop
        let pop_id = app.world_mut().spawn(Pop).id();

        app.add_systems(Update, initialize_pop_dialect);
        app.update();

        // The pop should have a personal dialect that incorporates the colony's prevailing slang
        let pop_dialect = app.world().get::<PersonalDialect>(pop_id).unwrap();
        assert!(pop_dialect.vocabulary.contains(&"rust".to_string()));
    }

    #[test]
    fn test_slang_decays_over_time_without_reinforcement() {
        let mut app = App::new();
        app.init_resource::<DialectManager>();
        app.insert_resource(SimulationTime::default());

        let mut dialect = app.world_mut().resource_mut::<DialectManager>();
        dialect.slang_dictionary.insert(
            "old_slang".to_string(),
            SlangEntry { usage: SlangUsage::Curse, weight: 0.5 }
        );

        // Advance time significantly
        app.world_mut().resource_mut::<SimulationTime>().ticks = 100_000;
        app.add_systems(Update, decay_slang_weight);
        app.update();

        let dialect = app.world().resource::<DialectManager>();
        // Weight should decrease, potentially falling below a threshold to be removed
        assert!(dialect.slang_dictionary.get("old_slang").map_or(true, |entry| entry.weight < 0.5));
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use bevy::utils::HashMap;
use crate::layer1::core::chronicle::{ChronicleEvent, EventImportance};
use crate::layer1::entities::pop::Pop;
use crate::shared::time::SimulationTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlangUsage {
    Curse,
    Praise,
    Greeting,
    Warning,
}

#[derive(Debug, Clone)]
pub struct SlangEntry {
    pub usage: SlangUsage,
    pub weight: f32, // How commonly it is used
}

#[derive(Resource, Default)]
pub struct DialectManager {
    pub slang_dictionary: HashMap<String, SlangEntry>,
}

#[derive(Component, Default)]
pub struct PersonalDialect {
    pub vocabulary: Vec<String>,
}

pub fn process_chronicle_events_for_dialect(
    mut events: EventReader<ChronicleEvent>,
    mut dialect_manager: ResMut<DialectManager>,
) {
    for event in events.read() {
        if event.importance == EventImportance::Major || event.importance == EventImportance::Legendary {
            let text_lower = event.text.to_lowercase();
            if text_lower.contains("fire") {
                dialect_manager.slang_dictionary.insert(
                    "fire".to_string(),
                    SlangEntry { usage: SlangUsage::Curse, weight: 1.0 },
                );
            }
            // Add other string matching rules for generating slang...
        }
    }
}

pub fn initialize_pop_dialect(
    mut commands: Commands,
    query: Query<Entity, Added<Pop>>,
    dialect_manager: Res<DialectManager>,
) {
    for entity in query.iter() {
        let mut vocab = Vec::new();
        for (word, entry) in dialect_manager.slang_dictionary.iter() {
            if entry.weight > 0.0 {
                vocab.push(word.clone());
            }
        }
        commands.entity(entity).insert(PersonalDialect { vocabulary: vocab });
    }
}

pub fn decay_slang_weight(
    mut dialect_manager: ResMut<DialectManager>,
    time: Res<SimulationTime>,
) {
    // Simplified decay logic based on time ticks
    if time.ticks % 1000 == 0 {
        for entry in dialect_manager.slang_dictionary.values_mut() {
            entry.weight -= 0.01;
            if entry.weight < 0.0 {
                entry.weight = 0.0;
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Extraction**: Extract slang generation logic into a separate `SlangGenerator` module that takes `ChronicleEvent` or `Memories` and applies a `Grammar` to produce varied slang (e.g., compounding words like "Fire-cursed").
- **Performance**: Instead of iterating over the entire dictionary for every new pop, use a shared `Arc` or reference to the current colony-wide `Dialect` to save memory, only diverging when a pop develops unique personal slang.
- **Integration**: Tie the `DialectManager` into the `NarrativeGenerator` and `Lore` systems so that UI dialog boxes and event notifications naturally use the generated slang terms.

## Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] A `ChronicleEvent` of high importance creates a corresponding `SlangEntry` in `DialectManager`.
- [ ] Newly spawned pops inherit prevalent slang from the `DialectManager` into their `PersonalDialect`.
- [ ] Slang weights decay over time as simulated by `decay_slang_weight`.

## Technical Guidance

- Use `bevy::utils::HashMap` for the `slang_dictionary`.
- Consider creating a generic event listener system that hooks into any major `ChronicleEvent` or `Memory` creation to seed the dialect engine.
- When applying slang to text generation, consider a text replacement pass in the UI rendering layer rather than storing the replaced text in the ECS state.

## Questions

*Builder: add questions here if spec is unclear.*

- **Architectural Contradictions:** `ChronicleEvent` lacks `event_type`, `severity`, and `description` fields. Instead, it uses `tick`, `year`, `text`, and `importance`. This makes the RED phase impossible to implement as written.

*Architect:* Addressed. The tests and implementation have been updated to use the actual fields `tick`, `year`, `text`, and `importance` on the `ChronicleEvent` struct.
