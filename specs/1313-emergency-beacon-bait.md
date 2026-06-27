# 1313: Emergency Beacon Bait

## Overview

You can deploy a fake "Distress Beacon" in deep space to attract "Heroes" (Friendly/Neutral ships) and "Scavengers" (Pirates). It allows setting traps for pirates, or ambushing friendly ships for loot. This introduces tension between Altruism (saving real distress) vs. Predation (faking it).

## Dependencies

- `018` — Faction System
- `024` — Ship Movement

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::map::StarSystemId;

    #[test]
    fn test_fake_beacon_attracts_ships() {
        let mut app = App::new();
        // Setup systems

        let target_system = StarSystemId(1);
        let beacon = app.world.spawn((FakeDistressBeacon { active: true }, Position { system: target_system })).id();
        let pirate_ship = app.world.spawn((Ship, Faction::Pirate, ShipMovement { destination: None })).id();

        // Act: Run beacon broadcast system and ship movement AI

        // Assert: Pirate ship's destination should update to target_system
    }

    #[test]
    fn test_fake_beacon_affects_reputation_if_discovered() {
        let mut app = App::new();
        // Setup systems

        let hero_faction = app.world.spawn((Faction::Hero, Reputation { with_player: 50 })).id();

        // Act: Run discovery event where Hero finds out beacon was fake

        // Assert: Reputation with Hero faction should decrease
    }

    #[test]
    fn test_deploying_beacon_consumes_resources() {
        let mut app = App::new();
        // Setup systems

        let mut inventory = Inventory::default();
        inventory.add(ItemType::Electronics, 10);
        let player = app.world.spawn((Player, inventory)).id();

        // Act: Deploy Fake Beacon

        // Assert: Player inventory should have less Electronics
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
#[derive(Component)]
pub struct FakeDistressBeacon {
    pub active: bool,
}

pub fn broadcast_fake_beacon(
    // Query for active FakeDistressBeacons
    // Query for Ships (Heroes/Scavengers) within range
    // Set Ship destination to Beacon location based on probability
) {
    // Implementation
}

pub fn handle_beacon_discovery(
    // Listen for ShipArrived events
    // If Ship arrives at FakeDistressBeacon and is Friendly
    // Trigger Reputation penalty
) {
    // Implementation
}
```

## REFACTOR Phase: Quality & Design

- **Performance**: Optimize ship querying using spatial partitioning or sector-based lookup instead of checking all ships against all beacons.
- **Integration**: Tie into the existing Narrative system so pops can gossip about setting traps for heroes.
- **Design**: Consider adding varying qualities of beacons (cheap ones only fool pirates, expensive ones fool heroes).

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Deploying a beacon attracts NPC ships to its location.
- [ ] Friendly ships discovering a fake beacon lower faction reputation.

## Technical Guidance

- Make sure to add cooldowns or costs to deploying beacons to prevent spamming.
- The probability of a ship responding should depend on their faction (Pirates are more likely to respond to a lone beacon in a dark sector).

## Questions

*Builder: add questions here if spec is unclear.*
