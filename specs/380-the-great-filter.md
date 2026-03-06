# 380: The Great Filter

## Overview

"The answer to the Fermi Paradox. The galaxy is a graveyard."

**The Great Filter** is a massive endgame mechanic for Layer 3. As a civilization advances its technology level, it triggers "Filter Events" (AI Uprising, Grey Goo, Mass Ascendancy). Surviving these apocalyptic challenges grants transcendent tech or traits. Failing wipes the civilization, leaving ruins for others to find.

This creates tension: Do you advance tech rapidly (risking a Filter event wiping you out) or stay primitive (safe but weak to external threats)?

## Dependencies

- `011` — Tech Tree Backend (for tracking technology advancement)
- `095` — System Generation (for spawning the ruins of failed civilizations)

## RED Phase: Tests First

Write these tests in `src/layer3/event/great_filter_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer3::tech::{CivilizationTechLevel, advance_tech_system};
    use crate::layer3::event::{FilterEvent, trigger_filter_events_system};

    #[test]
    fn test_high_tech_triggers_filter_event() {
        let mut world = World::new();
        world.init_resource::<Events<FilterEvent>>();

        let civ = world.spawn((
            CivilizationTechLevel { level: 90 }, // Approaching critical threshold
        )).id();

        // Simulate a tech advancement passing threshold 100
        world.send_event(crate::layer3::tech::TechAdvanceEvent { civ, amount: 15 });

        let mut schedule = Schedule::default();
        schedule.add_systems((advance_tech_system, trigger_filter_events_system).chain());
        schedule.run(&mut world);

        // Verify a Filter event was queued
        let filter_events = world.resource::<Events<FilterEvent>>();
        let mut reader = filter_events.get_reader();
        assert!(reader.read(filter_events).any(|e| e.civ == civ));
    }

    #[test]
    fn test_ruins_spawned_on_failed_filter() {
        let mut world = World::new();
        world.init_resource::<Events<crate::layer3::event::CivilizationWipeEvent>>();

        let civ = world.spawn((
            CivilizationTechLevel { level: 110 },
            crate::layer3::event::ActiveFilter { failed: true }, // Failed the test
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer3::event::process_failed_filters_system);
        schedule.run(&mut world);

        // A wipe event should fire, spawning ruins
        let wipe_events = world.resource::<Events<crate::layer3::event::CivilizationWipeEvent>>();
        let mut reader = wipe_events.get_reader();
        assert!(reader.read(wipe_events).any(|e| e.civ == civ));
    }
}
```

## GREEN Phase: Minimal Implementation

Implement this minimal logic in `src/layer3/event.rs`:

```rust
use bevy_ecs::prelude::*;
use crate::layer3::tech::CivilizationTechLevel;

#[derive(Event)]
pub struct FilterEvent {
    pub civ: Entity,
    pub filter_type: String, // e.g. "AI_UPRISING"
}

#[derive(Event)]
pub struct CivilizationWipeEvent {
    pub civ: Entity,
}

#[derive(Component)]
pub struct ActiveFilter {
    pub failed: bool,
}

pub fn trigger_filter_events_system(
    query: Query<(Entity, &CivilizationTechLevel), Changed<CivilizationTechLevel>>,
    mut events: EventWriter<FilterEvent>,
) {
    for (civ, tech) in query.iter() {
        if tech.level >= 100 {
            // MVP hardcoded trigger
            events.send(FilterEvent { civ, filter_type: "AI_UPRISING".to_string() });
        }
    }
}

pub fn process_failed_filters_system(
    query: Query<(Entity, &ActiveFilter)>,
    mut events: EventWriter<CivilizationWipeEvent>,
) {
    for (civ, filter) in query.iter() {
        if filter.failed {
            events.send(CivilizationWipeEvent { civ });
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- The MVP hardcodes `tech.level >= 100`. It should probabilistically roll for a Filter Event based on tech level, or trigger specific events tied to specific dangerous tech nodes (e.g., researching "General Artificial Intelligence").
- Add the Lore Master `TEMPLATES.md` triggers for these apocalyptic events so the chronicle accurately records a civilization falling to a Grey Goo scenario.
- Introduce actual mechanical tests for the Filter (combat waves, massive resource sinks, stability debuffs).

## Acceptance Criteria

- [ ] Reaching a specific tech threshold triggers a `FilterEvent` for a civilization.
- [ ] Failing an `ActiveFilter` results in a `CivilizationWipeEvent`, which converts their worlds into explorable ruins.
- [ ] Test coverage for the new module is >= 85%.
- [ ] `cargo test` and `cargo clippy -- -D warnings` pass.

## Technical Guidance

- This is an endgame system. The `FilterEvent` should completely disrupt normal play and force the player into a survival mode until the event is resolved or failed.
- The `CivilizationWipeEvent` should properly iterate over all planets owned by the `civ` and spawn Ruins/Artifacts.

## Questions

*Builder: add questions here if spec is unclear.*
