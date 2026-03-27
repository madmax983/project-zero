# 682 - Symbiotic Leviathans

## 1. Overview
Symbiotic Leviathans is a Layer 2 mechanic where players can lure and "tame" massive space fauna to act as organic cargo haulers or defense platforms. These beasts don't use conventional fuel, but they require massive amounts of specific Layer 1 resources (like exotic flora or entire herds of livestock) to remain docile. If their hunger is neglected, they will divert from trade routes to consume inhabited asteroid bases or colonies, triggering immediate wars and destruction.

## 2. Dependencies
- Layer 2 `Fleet` entities (extending or replacing them with Leviathans).
- Layer 1 `ColonyResources` (to track their specific, massive consumption).
- Layer 3 Diplomatic events (for when a Leviathan eats a neighbor's base).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::leviathans::{Leviathan, Tamed};
    use crate::layer1::resources::ColonyResources;

    #[test]
    fn test_leviathan_consumes_resources_and_remains_docile() {
        let mut app = App::new();
        app.add_systems(Update, process_leviathan_hunger);

        let colony = app.world_mut().spawn(ColonyResources {
            food: 10000, // Massive amount of livestock/flora
            ..default()
        }).id();

        let leviathan = app.world_mut().spawn((
            Leviathan { hunger: 500, home_base: colony },
            Tamed {},
        )).id();

        app.update();

        let resources = app.world().get::<ColonyResources>(colony).unwrap();
        let lev = app.world().get::<Leviathan>(leviathan).unwrap();

        assert_eq!(resources.food, 9500, "Leviathan should consume massive food per tick");
        assert_eq!(lev.hunger, 0, "Leviathan hunger should be satisfied");
        assert!(app.world().get::<Tamed>(leviathan).is_some(), "Leviathan remains tamed");
    }

    #[test]
    fn test_leviathan_goes_feral_when_starved() {
        let mut app = App::new();
        app.add_systems(Update, process_leviathan_hunger);

        let colony = app.world_mut().spawn(ColonyResources {
            food: 0, // No food
            ..default()
        }).id();

        let leviathan = app.world_mut().spawn((
            Leviathan { hunger: 500, home_base: colony },
            Tamed {},
        )).id();

        app.update();

        let lev = app.world().get::<Leviathan>(leviathan).unwrap();
        assert_eq!(lev.hunger, 500, "Hunger is not satisfied");
        assert!(app.world().get::<Tamed>(leviathan).is_none(), "Leviathan should lose Tamed component and go feral");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Leviathan {
    pub hunger: u32,
    pub home_base: Entity,
}

#[derive(Component)]
pub struct Tamed {}

#[derive(Component, Default)]
pub struct ColonyResources {
    pub food: u32,
}

pub fn process_leviathan_hunger(
    mut commands: Commands,
    mut colonies: Query<&mut ColonyResources>,
    mut leviathans: Query<(Entity, &mut Leviathan, Option<&Tamed>)>,
) {
    for (entity, mut leviathan, tamed_opt) in leviathans.iter_mut() {
        if tamed_opt.is_none() {
            continue; // Already feral
        }

        if let Ok(mut resources) = colonies.get_mut(leviathan.home_base) {
            if resources.food >= leviathan.hunger {
                resources.food -= leviathan.hunger;
                leviathan.hunger = 0;
            } else {
                // Not enough food to satisfy the beast
                commands.entity(entity).remove::<Tamed>();
            }
        } else {
            // Home base invalid, go feral
            commands.entity(entity).remove::<Tamed>();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring:** Give the Leviathans a "Hunger Meter" instead of a boolean toggle so the player has time to react.
- **Code Smells:** `home_base` assumes the Leviathan only eats from one place. In reality, it should pathfind to the nearest food source.
- **Performance:** Leviathan pathfinding (when feral) will require checking all Layer 2 nodes and Layer 1 colonies for the highest food concentration.
- **API Improvements:** Add an `Event` when a Leviathan goes feral to trigger the `Chronicle` entry and UI alerts.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Leviathans lose the `Tamed` tag and go feral when food is insufficient.

## 7. Technical Guidance
- **Code Structure:** Create `src/layer2/leviathans.rs` alongside fleets. Ensure they can be assigned trade routes via the existing UI.
- **Integration Points:** Link a feral Leviathan's consumption to the `OrbitalBombardmentEvent` or create a unique `LeviathanFeedingEvent` that damages Layer 1.

## 8. Questions
*Builder: add questions here if spec is unclear.*
