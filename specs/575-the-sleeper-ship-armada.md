# 575 The Sleeper Ship Armada

## 1. Overview
A Cross-layer (2 -> 1) moral dilemma mechanic. A massive "Sleeper Ship" arrives in Layer 2 orbit, packed with millions of frozen colonists. The player can choose to slowly awaken them (causing a massive population boom and food strain on Layer 1) or leave them frozen and strip-mine their ship for advanced precursor components. Strip-mining risks causing localized power failures on the ship, thawing survivors who may launch revenge strikes against the colony.

## 2. Dependencies
- Layer 2 `OrbitalEvent` system
- Layer 1 `Population` and `Stockpile` (food strain/components gain)
- Event dispatching for Player Decisions (`Awaken`, `StripMine`)
- `Combat` or `Raider` system for the revenge strike

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_awaken_sleeper_ship_spawns_pops() {
        let mut app = App::new();
        // Setup ...
        let ship = app.world.spawn(SleeperShip { population: 1000 }).id();
        let initial_pops = app.world.query::<&Pop>().iter(&app.world).count();

        app.world.send_event(SleeperShipDecision { target: ship, action: SleeperAction::Awaken });
        app.update();

        let new_pops = app.world.query::<&Pop>().iter(&app.world).count();
        assert!(new_pops > initial_pops);
        // Ship should be despawned or marked empty
        assert!(app.world.get::<SleeperShip>(ship).is_none());
    }

    #[test]
    fn test_strip_mine_sleeper_ship_grants_components_and_triggers_revenge() {
        let mut app = App::new();
        // Setup ...
        let ship = app.world.spawn(SleeperShip { population: 1000 }).id();
        let stockpile = app.world.spawn(Stockpile::new(100)).id();

        // Force revenge chance to 1.0 for test
        app.world.insert_resource(RevengeRNG { chance: 1.0 });

        app.world.send_event(SleeperShipDecision { target: ship, action: SleeperAction::StripMine });
        app.update();

        // Assert components gained
        let sp = app.world.get::<Stockpile>(stockpile).unwrap();
        assert!(sp.contains_item_type(ItemType::PrecursorComponent));

        // Assert revenge strike spawned
        let raiders = app.world.query::<&Raider>().iter(&app.world).count();
        assert!(raiders > 0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
pub enum SleeperAction {
    Awaken,
    StripMine,
}

#[derive(Event)]
pub struct SleeperShipDecision {
    pub target: Entity,
    pub action: SleeperAction,
}

#[derive(Component)]
pub struct SleeperShip {
    pub population: u32,
}

pub fn sleeper_ship_decision_system(
    mut events: EventReader<SleeperShipDecision>,
    mut commands: Commands,
    mut stockpiles: Query<&mut Stockpile>,
    rng: Res<RevengeRNG>, // Mock resource for testing
) {
    for event in events.read() {
        match event.action {
            SleeperAction::Awaken => {
                // Spawn Pops
                for _ in 0..10 { // Scaled down for simulation
                    commands.spawn(Pop);
                }
                commands.entity(event.target).despawn();
            }
            SleeperAction::StripMine => {
                // Grant components
                if let Some(mut stockpile) = stockpiles.iter_mut().next() {
                    stockpile.add(ItemType::PrecursorComponent);
                }

                // Check revenge
                if rng.chance >= 1.0 { // Simplified
                    commands.spawn((Raider, GridPosition { x: 0, y: 0 }));
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Hardcoding the spawn loop `for _ in 0..10` is bad. It should correctly scale based on the `SleeperShip.population` value and arrive via landing shuttles over time rather than instant manifestation.
- **Integration**: The `StripMine` action should be a continuous task, not an instant button click. It should generate `PrecursorComponent`s over several ticks, with a small probability of a `RevengeStrikeEvent` firing each tick it is mined.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Awakening the ship generates Layer 1 Pops.
- [ ] Strip-mining the ship generates PrecursorComponents but has a chance to spawn hostile Raiders.

## 7. Technical Guidance
- Implementing the delayed arrival of awakened Pops via an `IncomingShuttle` queue will prevent the simulation from stuttering due to instantly spawning thousands of entities.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
