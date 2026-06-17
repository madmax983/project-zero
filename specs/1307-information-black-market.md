# 1307: The Information Black Market

## Overview

Implementing "The Information Black Market" mechanic. If the player heavily restricts information flow (via censorship or propaganda) to artificially maintain high morale, an underground network forms where Pops trade real, unfiltered news of the galaxy at high resource costs. While this black market drains the colony's resources, it occasionally provides the player with critical early warnings of events in other sectors that official sensors missed, creating a strategic trade-off.

## Dependencies

- Existing Layer 1 Pop resource mechanics and Mood systems.
- Existing Layer 3 Galactic Events/Sensors.
- A mechanic to represent "Information Restriction" or "Censorship Level" (assumed to exist or will be added as part of this spec).

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_high_censorship_spawns_black_market() {
        let mut app = App::new();
        app.add_systems(Update, check_censorship_threshold);

        let colony = app.world_mut().spawn((
            Colony,
            CensorshipLevel(0.9), // High censorship
        )).id();

        // Act
        app.update();

        // Assert: The colony should now have an active InformationBlackMarket
        assert!(app.world().get::<InformationBlackMarket>(colony).is_some());
    }

    #[test]
    fn test_black_market_drains_resources() {
        let mut app = App::new();
        app.add_systems(Update, process_black_market_drain);

        let colony = app.world_mut().spawn((
            Colony,
            InformationBlackMarket { activity_level: 0.5 },
            ColonyResources { credits: 1000.0, ..default() },
        )).id();

        // Act
        app.update();

        // Assert: Resources should be reduced
        let resources = app.world().get::<ColonyResources>(colony).unwrap();
        assert!(resources.credits < 1000.0);
    }

    #[test]
    fn test_black_market_provides_early_warnings() {
        let mut app = App::new();
        app.add_event::<EarlyWarningEvent>();
        app.add_systems(Update, generate_black_market_intel);

        let colony = app.world_mut().spawn((
            Colony,
            InformationBlackMarket { activity_level: 1.0 }, // Max activity
        )).id();

        // Act: Run enough ticks to guarantee an event or mock the RNG
        // For testing, we assume a deterministic trigger when activity is 1.0
        app.update();

        // Assert: An early warning event should have been emitted
        let events = app.world().resource::<Events<EarlyWarningEvent>>();
        let mut reader = events.get_reader();
        assert!(reader.read(events).next().is_some());
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Colony;

#[derive(Component)]
pub struct CensorshipLevel(pub f32); // 0.0 to 1.0

#[derive(Component)]
pub struct InformationBlackMarket {
    pub activity_level: f32,
}

#[derive(Component, Default)]
pub struct ColonyResources {
    pub credits: f32,
    // other resources...
}

#[derive(Event)]
pub struct EarlyWarningEvent {
    pub sector_id: Entity,
    pub event_type: String,
}

pub fn check_censorship_threshold(
    mut commands: Commands,
    query: Query<(Entity, &CensorshipLevel), Without<InformationBlackMarket>>,
) {
    for (entity, censorship) in query.iter() {
        if censorship.0 > 0.75 {
            commands.entity(entity).insert(InformationBlackMarket {
                activity_level: (censorship.0 - 0.75) * 4.0, // Scales 0 to 1 based on excess censorship
            });
        }
    }
}

pub fn process_black_market_drain(
    mut query: Query<(&InformationBlackMarket, &mut ColonyResources)>,
) {
    for (market, mut resources) in query.iter_mut() {
        let drain_amount = market.activity_level * 10.0; // Base drain rate
        resources.credits -= drain_amount;
        if resources.credits < 0.0 {
            resources.credits = 0.0;
        }
    }
}

pub fn generate_black_market_intel(
    query: Query<&InformationBlackMarket>,
    mut event_writer: EventWriter<EarlyWarningEvent>,
) {
    for market in query.iter() {
        // Deterministic check for test, in reality this should use RNG
        if market.activity_level >= 1.0 {
            event_writer.send(EarlyWarningEvent {
                sector_id: Entity::PLACEHOLDER, // Mock data
                event_type: "Impending Invasion".to_string(),
            });
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **RNG for Intel**: The `generate_black_market_intel` system needs to use Bevy's RNG or a custom random resource to periodically emit events based on `activity_level`, rather than triggering deterministically.
- **Resource Generalization**: Instead of hardcoding `credits`, the black market could drain a variety of resources or specifically target luxury goods.
- **Dynamic Censorship**: Ensure that lowering the `CensorshipLevel` below the threshold gracefully removes the `InformationBlackMarket` component or reduces its `activity_level` to 0.

## Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] The `InformationBlackMarket` component is added when censorship is high.
- [ ] Resources are drained according to market activity.
- [ ] `EarlyWarningEvent` is occasionally emitted.

## Technical Guidance

### Components

- `CensorshipLevel`: Represents the current state of information restriction in a colony.
- `InformationBlackMarket`: Tracks the presence and intensity of the underground network.

### Systems

- `check_censorship_threshold`: Manages the addition/removal of the black market based on policy.
- `process_black_market_drain`: Handles the economic penalty of the black market.
- `generate_black_market_intel`: Handles the hidden benefit (early warnings).

### Integration Points

- Needs to integrate with Layer 3 sensors/events to make `EarlyWarningEvent` actionable (e.g., revealing an enemy fleet on the galaxy map before official sensors pick it up).
- UI should visually indicate the resource drain and the source of the intel.

## Questions

*Builder: add questions here if spec is unclear.*
