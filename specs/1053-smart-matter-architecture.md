# 1053: Smart Matter Architecture

## Overview

Buildings constructed with "Smart Matter" can change state based on conditions. "Smart Walls" become windows in sunlight or armored bunkers during raids. "Smart Floors" become conveyor belts. Requires constant power/CPU. This creates an adaptable but vulnerable base, as power failures can lock buildings in dangerous states (e.g., windows during a raid).

## Dependencies

- None specific, relies on base power/building system

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_smart_matter_state_change() {
        // Arrange: Setup app, add smart matter building, power grid, and raid event
        let mut app = App::new();
        app.add_event::<RaidEvent>();
        let entity = app.world.spawn((
            Building,
            SmartMatter { state: MatterState::Window },
            PowerReceiver { has_power: true },
        )).id();

        // Act: Trigger raid and run system
        app.world.send_event(RaidEvent);
        app.add_systems(Update, update_smart_matter_state);
        app.update();

        // Assert: State should change to Bunker
        let smart_matter = app.world.get::<SmartMatter>(entity).unwrap();
        assert_eq!(smart_matter.state, MatterState::Bunker);
    }

    #[test]
    fn test_smart_matter_power_failure() {
        // Arrange: Setup app, add smart matter building without power
        let mut app = App::new();
        app.add_event::<RaidEvent>();
        let entity = app.world.spawn((
            Building,
            SmartMatter { state: MatterState::Window },
            PowerReceiver { has_power: false },
        )).id();

        // Act: Trigger raid and run system
        app.world.send_event(RaidEvent);
        app.add_systems(Update, update_smart_matter_state);
        app.update();

        // Assert: State should remain Window due to lack of power
        let smart_matter = app.world.get::<SmartMatter>(entity).unwrap();
        assert_eq!(smart_matter.state, MatterState::Window);
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component, PartialEq, Debug)]
pub enum MatterState {
    Window,
    Bunker,
    Conveyor,
}

#[derive(Component)]
pub struct SmartMatter {
    pub state: MatterState,
}

#[derive(Component)]
pub struct Building;

#[derive(Component)]
pub struct PowerReceiver {
    pub has_power: bool,
}

#[derive(Event)]
pub struct RaidEvent;

pub fn update_smart_matter_state(
    mut query: Query<(&mut SmartMatter, &PowerReceiver)>,
    mut raid_events: EventReader<RaidEvent>,
) {
    let raid_active = !raid_events.is_empty();
    raid_events.clear(); // consume events

    for (mut smart_matter, power) in query.iter_mut() {
        if !power.has_power {
            continue;
        }

        if raid_active && smart_matter.state != MatterState::Bunker {
            smart_matter.state = MatterState::Bunker;
        } else if !raid_active && smart_matter.state != MatterState::Window {
            // Revert back to default state (simplified for green phase)
            smart_matter.state = MatterState::Window;
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- Introduce a more robust `Event` or `State` driven system for determining global conditions (Raid, Sunlight, etc.) rather than reading discrete events and clearing them.
- Abstract the transition logic to support dynamic rule sets per building.
- Add visual indicator components/systems when the matter state changes.
- Consider CPU constraints as an additional resource check alongside power.

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Smart matter transitions to bunker during raids when powered
- [ ] Smart matter fails to transition when unpowered

## Technical Guidance

- Ensure `update_smart_matter_state` runs after power distribution systems.
- You can store transition rules in a configuration resource instead of hardcoding them into the system.

## Questions

*Builder: add questions here if spec is unclear.*
