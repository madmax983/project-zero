# Spec 985: Propaganda Press

## 1. Overview
"Propaganda Press" enables players to control the narrative of their colony. By constructing a "Printing Press", a daily `Chronicle` is generated based on actual game events (like famines, deaths, or heroic deeds). Players can choose to redact specific negative logs. Pops reading the sanitized version receive mood buffs, but any Pops who actually witnessed the redacted event will gain a "Dissident" trait, potentially leading to factional revolts.

## 2. Dependencies
- Layer 1 Chronicle/Event logging system (`chronicle.rs` or `lore/mod.rs`).
- Layer 1 Pop Traits and Factions (`traits.rs`, `factions.rs`).
- Layer 1 Memory system (`memory.rs`).

## 3. RED Phase: Tests First

```rust
// tests/layer1_propaganda_press_tests.rs
use bevy::prelude::*;
use crate::layer1::social::propaganda::{DailyChronicle, RedactEvent, process_redactions_system, apply_propaganda_effects_system};
use crate::layer1::social::factions::{Trait, PopTraits};
use crate::layer1::memory::{Memories, MemoryEvent};
use crate::layer1::needs::NeedMorale;

#[test]
fn test_redacting_chronicle_hides_event() {
    let mut app = App::new();
    app.add_systems(Update, process_redactions_system);
    app.init_resource::<DailyChronicle>();
    app.init_resource::<Events<RedactEvent>>();

    // Setup an initial chronicle with a bad event
    let mut chronicle = app.world_mut().resource_mut::<DailyChronicle>();
    chronicle.entries.push("Starvation occurred in Sector 4".to_string());

    // Send a redact command
    app.world_mut().resource_mut::<Events<RedactEvent>>().send(RedactEvent {
        entry_index: 0,
    });

    app.update();

    // Verify the entry was redacted
    let updated_chronicle = app.world().resource::<DailyChronicle>();
    assert!(updated_chronicle.entries[0].contains("[REDACTED]"));
}

#[test]
fn test_witnesses_become_dissidents_when_reading_redacted_truth() {
    let mut app = App::new();
    app.add_systems(Update, apply_propaganda_effects_system);

    let mut chronicle = DailyChronicle::default();
    chronicle.entries.push("[REDACTED]".to_string());
    chronicle.redacted_truths.insert(0, "Starvation occurred in Sector 4".to_string());
    app.insert_resource(chronicle);

    // Spawn a pop who witnessed the event and is reading the paper
    let pop_entity = app.world_mut().spawn((
        PopTraits::default(),
        Memories { events: vec!["Starvation occurred in Sector 4".to_string()] },
        NeedMorale { value: 50.0, max: 100.0 }, // They will get angry, not happy
    )).id();

    app.update();

    // Verify pop gained Dissident trait
    let traits = app.world().entity(pop_entity).get::<PopTraits>().unwrap();
    assert!(traits.has_trait(Trait::Dissident));
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/social/propaganda.rs
use bevy::prelude::*;
use std::collections::HashMap;
use crate::layer1::social::factions::{Trait, PopTraits};
use crate::layer1::memory::Memories;
use crate::layer1::needs::NeedMorale;

#[derive(Resource, Default)]
pub struct DailyChronicle {
    pub entries: Vec<String>,
    pub redacted_truths: HashMap<usize, String>,
}

pub struct RedactEvent {
    pub entry_index: usize,
}

pub fn process_redactions_system(
    mut events: EventReader<RedactEvent>,
    mut chronicle: ResMut<DailyChronicle>,
) {
    for event in events.read() {
        if let Some(entry) = chronicle.entries.get_mut(event.entry_index) {
            let truth = entry.clone();
            *entry = "[REDACTED]".to_string();
            chronicle.redacted_truths.insert(event.entry_index, truth);
        }
    }
}

pub fn apply_propaganda_effects_system(
    chronicle: Res<DailyChronicle>,
    mut query: Query<(&mut PopTraits, &Memories, &mut NeedMorale)>,
) {
    for (mut traits, memories, mut morale) in query.iter_mut() {
        let mut is_dissident = false;

        // Check if they witnessed a redacted truth
        for (_, truth) in chronicle.redacted_truths.iter() {
            if memories.events.contains(truth) {
                traits.add_trait(Trait::Dissident);
                is_dissident = true;
                morale.value = (morale.value - 10.0).max(0.0);
            }
        }

        // If not a dissident, the sanitized paper boosts morale
        if !is_dissident && !chronicle.entries.is_empty() {
             morale.value = (morale.value + 5.0).min(morale.max);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Action Verification:** The `apply_propaganda_effects_system` currently applies effects to everyone. It should be constrained to Pops who are actively performing a `ReadNewspaper` action or are near a `PropagandaTerminal`.
- **Chronicle Generation:** Automate the ingestion of generic Layer 1 events (like `DeathEvent`, `CrimeEvent`, `HeroicActEvent`) into the `DailyChronicle` at the start of each simulation day.
- **Dissident Escalation:** Dissidents shouldn't just get angry; they should gradually form a `RebelFaction` and actively attempt to sabotage the "Printing Press" building or distribute underground un-redacted pamphlets.

## 6. Acceptance Criteria (Testable!)
- [ ] `cargo test` returns 0 failures, including `test_redacting_chronicle_hides_event`.
- [ ] `cargo clippy --all-targets -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new `propaganda.rs` module.
- [ ] The `DailyChronicle` resource properly stores original truths when entries are redacted.
- [ ] Pops with matching memories correctly gain the `Trait::Dissident` when exposed to redacted chronicles.

## 7. Technical Guidance
- Create a new module `src/layer1/social/propaganda.rs`.
- Ensure you initialize `Events<RedactEvent>` and the `DailyChronicle` resource in `setup.rs` or the relevant integration test bootstraps.
- Keep the `Memories` struct string comparison robust; you may want to use explicit `EventId` enums rather than pure strings in a production environment to avoid fragile matching.

## 8. Questions
*Builder: add questions here if spec is unclear.*
