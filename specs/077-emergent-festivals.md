# 077: Emergent Festivals

## Overview

The colony should celebrate its history. **Emergent Festivals** occur when the current date matches the anniversary of a significant past event recorded in the Chronicle.
- **FestivalState**: Tracks active festival.
- **Mechanic**: Every day (or season change), check if a "Legendary" or "Major" event happened on this day in a previous year.
- **Effect**: During a festival, Pops gain a Morale boost and potentially a "Partying" thought.

This transforms the procedural history (Chronicle) into gameplay effects, making each playthrough unique. A colony that had a "Great Famine" might celebrate "Survival Day" with a feast.

## Dependencies

- `010` — Chronicle System (provides `ChronicleEvent`, `EventImportance`, `TICKS_PER_YEAR`)
- `031` — Pop Morale (provides `MoodModifier`)
- `027` — Seasonal Rhythms (provides `TICKS_PER_YEAR`)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/festivals.rs - Test module at end of file

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::chronicle::{Chronicle, EventImportance, TICKS_PER_YEAR};
    use crate::shared::time::SimulationTime;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_festival_state_default() {
        let state = FestivalState::default();
        assert!(state.active_festival.is_none());
    }

    #[test]
    fn test_check_for_festivals_no_events() {
        let mut world = World::new();
        world.insert_resource(FestivalState::default());
        world.insert_resource(Chronicle::default());
        world.insert_resource(SimulationTime { tick: 100, ..Default::default() });
        // Add log resource if needed by system
        world.insert_resource(crate::shared::log::MessageLog::default());

        check_for_festivals_system(&mut world);

        let state = world.resource::<FestivalState>();
        assert!(state.active_festival.is_none());
    }

    #[test]
    fn test_check_for_festivals_anniversary() {
        let mut world = World::new();
        world.insert_resource(FestivalState::default());
        world.insert_resource(crate::shared::log::MessageLog::default());

        let mut chronicle = Chronicle::default();
        // Event happened at tick 100 (Year 1)
        chronicle.add_event(100, "Founding Day".to_string(), EventImportance::Legendary);
        world.insert_resource(chronicle);

        // Current time: Tick 100 + TICKS_PER_YEAR (Year 2, same day)
        world.insert_resource(SimulationTime {
            tick: 100 + TICKS_PER_YEAR,
            ..Default::default()
        });

        check_for_festivals_system(&mut world);

        let state = world.resource::<FestivalState>();
        assert!(state.active_festival.is_some());
        let festival = state.active_festival.as_ref().unwrap();
        assert_eq!(festival.name, "Founding Day Festival");
        assert_eq!(festival.end_tick, 100 + TICKS_PER_YEAR + FESTIVAL_DURATION);
    }

    #[test]
    fn test_check_for_festivals_ignores_minor_events() {
        let mut world = World::new();
        world.insert_resource(FestivalState::default());
        world.insert_resource(crate::shared::log::MessageLog::default());

        let mut chronicle = Chronicle::default();
        chronicle.add_event(100, "Ate a berry".to_string(), EventImportance::Minor);
        world.insert_resource(chronicle);

        world.insert_resource(SimulationTime {
            tick: 100 + TICKS_PER_YEAR,
            ..Default::default()
        });

        check_for_festivals_system(&mut world);

        let state = world.resource::<FestivalState>();
        assert!(state.active_festival.is_none(), "Minor events should not trigger festivals");
    }

    #[test]
    fn test_festival_ends_after_duration() {
        let mut world = World::new();

        // Festival ending at tick 200
        let festival = Festival {
            name: "Test Festival".to_string(),
            original_event_tick: 0,
            end_tick: 200,
        };

        world.insert_resource(FestivalState {
            active_festival: Some(festival),
        });

        // Current time: 201
        world.insert_resource(SimulationTime { tick: 201, ..Default::default() });
        world.insert_resource(crate::shared::log::MessageLog::default()); // Depending on if end logs message

        // Run system (we might need a separate cleanup system or check_for_festivals handles it)
        festival_lifecycle_system(&mut world);

        let state = world.resource::<FestivalState>();
        assert!(state.active_festival.is_none());
    }

    #[test]
    fn test_apply_festival_morale_effect() {
        let mut world = World::new();

        // Active festival
        world.insert_resource(FestivalState {
            active_festival: Some(Festival {
                name: "Party".to_string(),
                original_event_tick: 0,
                end_tick: 1000,
            }),
        });

        // Pop with morale
        let pop = world.spawn(crate::layer1::pop::Pop).id();
        // Assuming we have a Morale component or similar.
        // Spec 031 uses `Mood` or `Morale`. Let's assume `Mood`.
        // If `Mood` is complex, we might check for `Memory` insertion or direct modifier query.

        // For this test, let's verify the *modifier function* returns a bonus.
        // This follows the pattern from Edicts (054).

        let bonus = get_festival_morale_modifier(&world.resource::<FestivalState>());
        assert!(bonus > 0.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Resource and Structs

```rust
// src/layer1/festivals.rs

use bevy_ecs::prelude::*;
use crate::layer1::chronicle::{Chronicle, EventImportance, TICKS_PER_YEAR};
use crate::shared::time::SimulationTime;
use crate::shared::log::MessageLog;

pub const FESTIVAL_DURATION: u64 = 500; // Half a day? Or 100 ticks? Let's say 200.
// TICKS_PER_YEAR is 1000. So 200 is 20% of a year. Maybe too long.
// Let's set it to 100 (10% of year).

#[derive(Debug, Clone)]
pub struct Festival {
    pub name: String,
    pub original_event_tick: u64,
    pub end_tick: u64,
}

#[derive(Resource, Default)]
pub struct FestivalState {
    pub active_festival: Option<Festival>,
}

/// Helper to get current morale modifier from festivals.
pub fn get_festival_morale_modifier(state: &FestivalState) -> f32 {
    if state.active_festival.is_some() {
        10.0 // +10 Mood
    } else {
        0.0
    }
}
```

### 2. Implement Systems

```rust
// src/layer1/festivals.rs

pub fn check_for_festivals_system(
    mut state: ResMut<FestivalState>,
    chronicle: Res<Chronicle>,
    time: Res<SimulationTime>,
    mut log: ResMut<MessageLog>,
) {
    let current_tick = time.tick;

    // Do not override existing festival
    if let Some(festival) = &state.active_festival {
        if current_tick >= festival.end_tick {
            // Festival ended
            log.add(format!("The {} has ended.", festival.name));
            state.active_festival = None;
        }
        return;
    }

    // Check for anniversaries
    // We only check once per day/tick? To avoid scanning 1000 events every tick?
    // Optimization: Check only if current_tick % TICKS_PER_YEAR == event.tick % TICKS_PER_YEAR
    // But we iterate all events. With < 1000 events it's fast.

    // We only want to trigger if current_tick is EXACTLY the anniversary.
    // And year > event.year

    for event in &chronicle.events {
        if event.importance == EventImportance::Minor {
            continue;
        }

        // Check if today is the anniversary
        // (current_tick - event.tick) % TICKS_PER_YEAR == 0
        // AND current_tick > event.tick (it's in the past)
        if current_tick > event.tick && (current_tick - event.tick) % TICKS_PER_YEAR == 0 {
            // Found one!
            let name = format!("{} Festival", event.text.chars().take(20).collect::<String>().trim());

            state.active_festival = Some(Festival {
                name: name.clone(),
                original_event_tick: event.tick,
                end_tick: current_tick + 100, // Duration 100 ticks
            });

            log.add(format!("Today we celebrate {}!", name));
            break; // Only one festival at a time
        }
    }
}

// Separate lifecycle system if needed, but combined above is simpler for MVP.
// We can split if testing requires it (test_festival_ends_after_duration).
// Let's split for TDD clarity.

pub fn festival_lifecycle_system(
    mut state: ResMut<FestivalState>,
    time: Res<SimulationTime>,
    mut log: ResMut<MessageLog>,
) {
    if let Some(festival) = &state.active_festival {
        if time.tick >= festival.end_tick {
            log.add(format!("The {} has ended.", festival.name));
            state.active_festival = None;
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Optimization**: Don't scan the entire chronicle every tick.
  - *Better approach*: Pre-calculate "festival calendar" when an event is added, or scan only at start of day.
  - *MVP*: Scanning 100 events is fine.
- **Naming**: "Founding Day Festival" is better than "Colony founded. The... Festival".
  - Need a way to extract a short title from the event text, or store a `title` field in `ChronicleEvent`.
- **UI**: Display "FESTIVAL!" in the top bar.

## Acceptance Criteria

- [ ] `FestivalState` resource exists.
- [ ] `check_for_festivals_system` triggers a festival on the anniversary of Legendary/Major events.
- [ ] Festival ends after set duration.
- [ ] `get_festival_morale_modifier` returns a positive value during festivals.
- [ ] Minor events do NOT trigger festivals.
- [ ] Only one festival active at a time.

## Technical Guidance

- In `layer1/mod.rs`, register the module and systems.
- In `main.rs`, add `check_for_festivals_system` to the schedule (e.g., in `Plan` or `Think` phase).
- Ensure `MessageLog` is available (it's in `shared::log`).
