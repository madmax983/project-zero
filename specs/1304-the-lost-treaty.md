# 1304: The Lost Treaty

## Overview

A piece of ancient bureaucracy that stops a fleet. Players can discover an ancient legal claim ("The Lost Treaty") to a specific sector. Enforcing this claim grants a "Legitimacy" buff (increasing diplomatic standing globally) but angers the current occupants of that sector. Ignoring the treaty avoids immediate conflict but incurs a minor legitimacy penalty, making the player look weak. This introduces a tension between Law (Legitimacy) and Reality (War).

## Dependencies

- Layer 3 scaffolding (Galaxy/Diplomacy layer if existing, otherwise this builds upon it)

## RED Phase: Tests First

```rust
// src/layer3/diplomacy/tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_discover_treaty_creates_dilemma() {
        // Arrange
        let mut app = App::new();
        app.add_event::<DiscoverTreatyEvent>();
        app.add_systems(Update, handle_treaty_discovery);

        let player_faction = app.world_mut().spawn(Faction { legitimacy: 50 }).id();
        let target_sector = app.world_mut().spawn(Sector { occupied_by: Some(player_faction) }).id();

        // Act
        app.world_mut().resource_mut::<Events<DiscoverTreatyEvent>>().send(DiscoverTreatyEvent {
            target_sector,
            discovering_faction: player_faction,
        });
        app.update();

        // Assert
        // The treaty dilemma should now be tracked in the player's pending events or a global resource
        let dilemma_query = app.world_mut().query::<&TreatyDilemma>().get_single(app.world());
        assert!(dilemma_query.is_ok(), "Treaty dilemma entity should be spawned upon discovery");
        assert_eq!(dilemma_query.unwrap().target_sector, target_sector);
    }

    #[test]
    fn test_enforce_treaty_grants_legitimacy_angers_occupant() {
        // Arrange
        let mut app = App::new();
        app.add_event::<EnforceTreatyEvent>();
        app.add_systems(Update, handle_enforce_treaty);

        let player_faction = app.world_mut().spawn(Faction { legitimacy: 50 }).id();
        let occupant_faction = app.world_mut().spawn(Faction { legitimacy: 50 }).id();

        // Add diplomatic relation
        app.world_mut().spawn(DiplomaticRelation {
            faction_a: player_faction,
            faction_b: occupant_faction,
            standing: 0,
        });

        // Act
        app.world_mut().resource_mut::<Events<EnforceTreatyEvent>>().send(EnforceTreatyEvent {
            enforcing_faction: player_faction,
            occupant_faction,
        });
        app.update();

        // Assert
        let p_faction = app.world_mut().get::<Faction>(player_faction).unwrap();
        assert!(p_faction.legitimacy > 50, "Enforcing should increase legitimacy");

        let relation = app.world_mut().query::<&DiplomaticRelation>().iter(app.world()).next().unwrap();
        assert!(relation.standing < 0, "Enforcing should anger the occupant");
    }

    #[test]
    fn test_ignore_treaty_reduces_legitimacy() {
        // Arrange
        let mut app = App::new();
        app.add_event::<IgnoreTreatyEvent>();
        app.add_systems(Update, handle_ignore_treaty);

        let player_faction = app.world_mut().spawn(Faction { legitimacy: 50 }).id();

        // Act
        app.world_mut().resource_mut::<Events<IgnoreTreatyEvent>>().send(IgnoreTreatyEvent {
            ignoring_faction: player_faction,
        });
        app.update();

        // Assert
        let p_faction = app.world_mut().get::<Faction>(player_faction).unwrap();
        assert!(p_faction.legitimacy < 50, "Ignoring treaty should reduce legitimacy");
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
// src/layer3/diplomacy/mod.rs

use bevy::prelude::*;

#[derive(Component)]
pub struct Faction {
    pub legitimacy: i32,
}

#[derive(Component)]
pub struct Sector {
    pub occupied_by: Option<Entity>,
}

#[derive(Component)]
pub struct TreatyDilemma {
    pub target_sector: Entity,
    pub discovering_faction: Entity,
}

#[derive(Component)]
pub struct DiplomaticRelation {
    pub faction_a: Entity,
    pub faction_b: Entity,
    pub standing: i32,
}

#[derive(Event)]
pub struct DiscoverTreatyEvent {
    pub target_sector: Entity,
    pub discovering_faction: Entity,
}

#[derive(Event)]
pub struct EnforceTreatyEvent {
    pub enforcing_faction: Entity,
    pub occupant_faction: Entity,
}

#[derive(Event)]
pub struct IgnoreTreatyEvent {
    pub ignoring_faction: Entity,
}

pub fn handle_treaty_discovery(
    mut commands: Commands,
    mut events: EventReader<DiscoverTreatyEvent>,
) {
    for event in events.read() {
        commands.spawn(TreatyDilemma {
            target_sector: event.target_sector,
            discovering_faction: event.discovering_faction,
        });
    }
}

pub fn handle_enforce_treaty(
    mut events: EventReader<EnforceTreatyEvent>,
    mut factions: Query<&mut Faction>,
    mut relations: Query<&mut DiplomaticRelation>,
) {
    for event in events.read() {
        if let Ok(mut faction) = factions.get_mut(event.enforcing_faction) {
            faction.legitimacy += 10;
        }

        for mut relation in relations.iter_mut() {
            if (relation.faction_a == event.enforcing_faction && relation.faction_b == event.occupant_faction) ||
               (relation.faction_b == event.enforcing_faction && relation.faction_a == event.occupant_faction) {
                relation.standing -= 50;
            }
        }
    }
}

pub fn handle_ignore_treaty(
    mut events: EventReader<IgnoreTreatyEvent>,
    mut factions: Query<&mut Faction>,
) {
    for event in events.read() {
        if let Ok(mut faction) = factions.get_mut(event.ignoring_faction) {
            faction.legitimacy -= 5;
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- Introduce a constant or config for the legitimacy bonuses/penalties (+10 / -5) and standing penalty (-50) to make balancing easier.
- Instead of raw strings or basic structs, integrate with Layer 3 `Chronicle` event generation. When a treaty is enforced or ignored, it should emit an `AddChronicleEvent` leveraging a lore template.
- Ensure `DiplomaticRelation` handles cases where relations don't exist yet between the two factions (maybe implicitly spawn one with default standing).

## Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified (Enforcing increases legitimacy and lowers standing; Ignoring decreases legitimacy).

## Technical Guidance
- If `Faction`, `Sector`, or `DiplomaticRelation` already exist in the codebase, use those existing structures instead of creating new ones.
- The `handle_treaty_discovery` might want to tie the spawned `TreatyDilemma` to the user interface so the player can actually click a button to Enforce or Ignore.
- Consider what happens if the sector is unoccupied when the treaty is found.

## Questions
*Builder: add questions here if spec is unclear.*
