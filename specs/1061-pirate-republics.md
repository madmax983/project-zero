# 1061: Pirate Republics

## Overview

Crime eventually evolves into government. This feature implements "Pirate Republics" where pirates aren't just random spawns, but belong to "Havens" (Hidden Bases). Successful raids upgrade Havens, accumulating wealth and power. Eventually, a highly upgraded Haven will transition into a legitimate Pirate Republic Faction, engaging in diplomacy, demanding tribute, or offering protection to the colony.

## Dependencies

- None

## RED Phase: Tests First

```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_raid_success_increases_haven_wealth() {
        // Arrange
        let mut app = App::new();
        app.add_event::<RaidSuccessEvent>();
        app.add_systems(Update, process_raid_success_system);

        let haven = app.world_mut().spawn((
            PirateHaven { level: 1, wealth: 100 },
        )).id();

        // Act
        app.world_mut().send_event(RaidSuccessEvent {
            haven_entity: haven,
            loot_value: 50,
        });
        app.update();

        // Assert
        let haven_comp = app.world().get::<PirateHaven>(haven).unwrap();
        assert_eq!(haven_comp.wealth, 150);
    }

    #[test]
    fn test_haven_upgrades_when_wealth_threshold_met() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, haven_upgrade_system);

        let haven = app.world_mut().spawn((
            PirateHaven { level: 1, wealth: 550 },
        )).id();

        // Act
        app.update();

        // Assert
        let haven_comp = app.world().get::<PirateHaven>(haven).unwrap();
        assert_eq!(haven_comp.level, 2);
        assert_eq!(haven_comp.wealth, 50); // Wealth consumed for upgrade (threshold 500)
    }

    #[test]
    fn test_max_level_haven_becomes_republic() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, haven_to_republic_system);

        let haven = app.world_mut().spawn((
            PirateHaven { level: 5, wealth: 1000 },
        )).id();

        // Act
        app.update();

        // Assert
        let haven_comp = app.world().get::<PirateHaven>(haven);
        assert!(haven_comp.is_none());

        let republic = app.world().get::<PirateRepublic>(haven);
        assert!(republic.is_some());

        let faction = app.world().get::<Faction>(haven);
        assert!(faction.is_some());
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct PirateHaven {
    pub level: u32,
    pub wealth: u32,
}

#[derive(Component)]
pub struct PirateRepublic;

#[derive(Component)]
pub struct Faction {
    pub name: String,
    pub relationship_score: i32,
}

#[derive(Event)]
pub struct RaidSuccessEvent {
    pub haven_entity: Entity,
    pub loot_value: u32,
}

const UPGRADE_THRESHOLD_BASE: u32 = 500;
const MAX_HAVEN_LEVEL: u32 = 5;

pub fn process_raid_success_system(
    mut events: EventReader<RaidSuccessEvent>,
    mut havens: Query<&mut PirateHaven>,
) {
    for event in events.read() {
        if let Ok(mut haven) = havens.get_mut(event.haven_entity) {
            haven.wealth += event.loot_value;
        }
    }
}

pub fn haven_upgrade_system(
    mut havens: Query<&mut PirateHaven>,
) {
    for mut haven in havens.iter_mut() {
        let upgrade_cost = UPGRADE_THRESHOLD_BASE * haven.level;
        if haven.wealth >= upgrade_cost && haven.level < MAX_HAVEN_LEVEL {
            haven.wealth -= upgrade_cost;
            haven.level += 1;
        }
    }
}

pub fn haven_to_republic_system(
    mut commands: Commands,
    havens: Query<(Entity, &PirateHaven)>,
) {
    for (entity, haven) in havens.iter() {
        if haven.level >= MAX_HAVEN_LEVEL {
            commands.entity(entity)
                .remove::<PirateHaven>()
                .insert(PirateRepublic)
                .insert(Faction {
                    name: "New Pirate Republic".to_string(),
                    relationship_score: -50, // Initially hostile
                });
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- Ensure the logic scaling upgrade costs is balanced.
- Pirate Republics should integrate closely with the existing `Diplomacy` and `Faction` modules rather than re-inventing relationships.
- Add notifications or `HistoryEvent` integration so the colony knows when a local Haven transitions into a Republic.
- Consider what happens when raids fail - do havens lose wealth?

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Havens correctly absorb wealth from successful raids via `RaidSuccessEvent`.
- [ ] Havens upgrade when crossing wealth thresholds.
- [ ] Havens reaching the max level transition into legitimate `PirateRepublic` factions capable of diplomacy.

## Technical Guidance

- Systems could be placed in `src/layer2/governance/piracy.rs` or an equivalent logical grouping for Layer 2 entities.
- Verify `Faction` creation meets any initialization requirements set by the broader system if using an existing type.

## Questions

*Builder: add questions here if spec is unclear.*
