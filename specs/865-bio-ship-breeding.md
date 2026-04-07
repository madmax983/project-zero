# Specification: 865 Bio-Ship Breeding

## 1. Overview
**Layer:** 2
**Fantasy:** Growing a fleet instead of building it.
**Mechanic:** Some factions use organic ships. Instead of needing Ore and Factories, these ships require Biomass and "Gestation Vats". They have high regeneration but require constant food upkeep; if they starve, they go feral and attack indiscriminately.

## 2. Dependencies
- Layer 2 Ships (`src/layer2/ship.rs`)
- ColonyResources (Biomass/Food)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_bioship_consumes_food_upkeep() {
        let mut app = App::new();
        app.add_plugins(BioShipPlugin);

        let ship = app.world_mut().spawn((
            BioShip,
            HungerState { current: 100, max: 100, decay_rate: 10 },
            FeralState::Docile,
        )).id();

        // Simulate tick
        app.world_mut().insert_resource(Time::new_with(bevy::utils::Duration::from_secs(1)));
        app.update();

        let hunger = app.world().get::<HungerState>(ship).unwrap();
        assert_eq!(hunger.current, 90, "BioShip hunger should decay over time");
    }

    #[test]
    fn test_starving_bioship_goes_feral() {
        let mut app = App::new();
        app.add_plugins(BioShipPlugin);

        let ship = app.world_mut().spawn((
            BioShip,
            HungerState { current: 5, max: 100, decay_rate: 10 },
            FeralState::Docile,
        )).id();

        app.world_mut().insert_resource(Time::new_with(bevy::utils::Duration::from_secs(1)));
        app.update();

        let hunger = app.world().get::<HungerState>(ship).unwrap();
        assert_eq!(hunger.current, 0, "Hunger should not drop below 0");

        let feral_state = app.world().get::<FeralState>(ship).unwrap();
        assert_eq!(*feral_state, FeralState::Feral, "Starving BioShip should become feral");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct BioShip;

#[derive(Component)]
pub struct HungerState {
    pub current: i32,
    pub max: i32,
    pub decay_rate: i32,
}

#[derive(Component, PartialEq, Debug)]
pub enum FeralState {
    Docile,
    Feral,
}

pub struct BioShipPlugin;

impl Plugin for BioShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_bioship_hunger_system);
    }
}

fn handle_bioship_hunger_system(
    mut ships: Query<(&mut HungerState, &mut FeralState), With<BioShip>>,
) {
    for (mut hunger, mut feral_state) in ships.iter_mut() {
        hunger.current = (hunger.current - hunger.decay_rate).max(0);

        if hunger.current == 0 {
            *feral_state = FeralState::Feral;
        } else {
            *feral_state = FeralState::Docile; // Regains composure if fed
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- A feral BioShip should immediately detach from player control. Ensure that the Faction/Ownership component is updated when it goes feral.
- Implement a feeding system where the ship can consume `ColonyResources` (Biomass) from its current system node to restore `HungerState`.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the module.
- [ ] BioShip hunger decays correctly per tick.
- [ ] BioShips transition to `FeralState::Feral` when hunger reaches 0.

## 7. Technical Guidance
- Feeding could be automatic if docked at a friendly station with surplus Biomass, or manual via a command.
- Feral ships should target the nearest entity (friend or foe) to attempt to "eat" them (damaging hull to restore their own hunger).

## 8. Questions
*Builder: Add questions here if spec is unclear.*
