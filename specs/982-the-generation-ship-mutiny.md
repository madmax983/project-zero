# Spec 982: The Generation Ship Mutiny

## 1. Overview
"The Generation Ship Mutiny" simulates the extreme long-term risks of distant colonization. A Layer 2 Generation Ship takes centuries to arrive. Players can "zoom in" and manage the ship as a tiny Layer 1 colony. Over time, original mission parameters are forgotten, and descendants may form radical factions (e.g., ship-worshippers who refuse to land or mutineers who demand to settle on the nearest barren asteroid).

## 2. Dependencies
- Layer 2 Fleet Movement.
- Layer 1 Colony simulation / Faction systems.
- Cross-layer zooming/state management.

## 3. RED Phase: Tests First

```rust
// tests/generation_ship_mutiny_tests.rs
use bevy::prelude::*;
use crate::layer1::social::factions::{Faction, Radicalization};
use crate::layer2::ships::{GenerationShip, Destination};
use crate::cross_layer::{
    MutinyEvent, generation_ship_radicalization_system,
    evaluate_mutiny_system
};

#[test]
fn test_generation_ship_generates_radical_factions_over_time() {
    let mut app = App::new();
    app.add_systems(Update, generation_ship_radicalization_system);

    // Spawn a Generation Ship entity
    let ship_entity = app.world_mut().spawn((
        GenerationShip { age_in_years: 100 },
        Destination { target: Entity::PLACEHOLDER }
    )).id();

    // Spawn a faction aboard the ship
    let faction_entity = app.world_mut().spawn((
        Faction { name: "Original Mission".to_string(), host_ship: ship_entity },
        Radicalization { level: 0.0 }
    )).id();

    app.update();

    let radicalization = app.world().get::<Radicalization>(faction_entity).unwrap();
    assert!(radicalization.level > 0.0, "Factions on generation ships should radicalize over time.");
}

#[test]
fn test_high_radicalization_triggers_mutiny_event() {
    let mut app = App::new();
    app.add_event::<MutinyEvent>();
    app.add_systems(Update, evaluate_mutiny_system);

    let ship_entity = app.world_mut().spawn(GenerationShip { age_in_years: 200 }).id();

    // Spawn a highly radical faction
    app.world_mut().spawn((
        Faction { name: "Ship Worshippers".to_string(), host_ship: ship_entity },
        Radicalization { level: 100.0 } // threshold met
    ));

    app.update();

    let events = app.world().resource::<Events<MutinyEvent>>();
    let mut reader = events.get_reader();
    let mutiny_events: Vec<_> = reader.read(events).collect();

    assert_eq!(mutiny_events.len(), 1, "High radicalization should trigger a MutinyEvent.");
    assert_eq!(mutiny_events[0].ship, ship_entity);
}
```

## 4. GREEN Phase: Minimal Implementation
- Define `GenerationShip` and `Radicalization` components.
- `MutinyEvent` should signal a change in the ship's behavior or destination.
- `generation_ship_radicalization_system`: Iterates over factions residing on `GenerationShip`s. Slowly increases `Radicalization.level` based on ship age, low morale, or lack of progress.
- `evaluate_mutiny_system`: Checks `Radicalization.level` against a threshold. If crossed, emit `MutinyEvent`.
- When a `MutinyEvent` fires, standard logic should perhaps change the ship's `Destination` to a nearby invalid target or halt it completely.

## 5. REFACTOR Phase: Quality & Design
- **Integration:** Hook the `MutinyEvent` into the actual Layer 2 navigation system so the ship refuses player move orders once mutinied.
- **Narrative Hooks:** Provide chronicles or logs when the mutiny occurs so the player understands *why* the ship is stopping.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Coverage >= 85%.
- [ ] Factions on generation ships radicalize over time.
- [ ] Radical factions cause mutinies that alter the ship's path or behavior.

## 7. Technical Guidance
- **Host linking:** Factions usually live on Layer 1 colonies. Ensure they can be cleanly parented or linked to a Layer 2 `GenerationShip` entity without breaking global queries.

## 8. Questions
*Builder: Add questions here if linking Factions to Ships is technically difficult in the current architecture.*
