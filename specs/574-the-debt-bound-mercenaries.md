# 574 The Debt-Bound Mercenaries

## 1. Overview
A Cross-layer (3 -> 2 -> 1) crisis mechanic. During a desperate siege, players can hire an overwhelming Layer 3 mercenary fleet to defend their Layer 2 system for an astronomical daily upkeep. If the colony runs out of credits while the fleet is stationed, the mercenaries don't leave; they land on Layer 1 and forcefully seize the colony's most profitable industrial zones to extract payment, turning the capital into their base of operations.

## 2. Dependencies
- Layer 3 `Diplomacy` / `Fleet` entities
- Layer 2 `OrbitalSystem` (mercenaries stationed)
- Layer 1 `Credits` resource, `Building` / `Zone` entities
- `HostileTakeover` event system

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_mercenary_fleet_drains_credits_daily() {
        let mut app = App::new();
        // Setup ...
        app.world.insert_resource(Credits { amount: 10000 });
        app.world.spawn(MercenaryFleet { daily_upkeep: 2000, state: FleetState::Orbiting });

        app.update(); // Run daily upkeep system

        assert_eq!(app.world.resource::<Credits>().amount, 8000);
    }

    #[test]
    fn test_mercenary_fleet_seizes_buildings_on_bankruptcy() {
        let mut app = App::new();
        // Setup ...
        app.world.insert_resource(Credits { amount: 0 });
        let fleet = app.world.spawn(MercenaryFleet { daily_upkeep: 2000, state: FleetState::Orbiting }).id();

        let fusion_plant = app.world.spawn((Building::FusionPlant, Faction::Player)).id();

        app.update(); // Run bankruptcy check system

        // Assert fleet state changed to Landed/Occupying
        assert_eq!(app.world.get::<MercenaryFleet>(fleet).unwrap().state, FleetState::Occupying);
        // Assert building faction changed to Mercenary
        assert_eq!(*app.world.get::<Faction>(fusion_plant).unwrap(), Faction::Mercenary);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum FleetState {
    Orbiting,
    Occupying,
}

#[derive(Component)]
pub struct MercenaryFleet {
    pub daily_upkeep: u32,
    pub state: FleetState,
}

#[derive(Component, PartialEq, Debug)]
pub enum Faction {
    Player,
    Mercenary,
}

pub fn mercenary_upkeep_system(
    mut credits: ResMut<Credits>,
    mut fleets: Query<&mut MercenaryFleet>,
    mut buildings: Query<&mut Faction, With<Building>>,
) {
    for mut fleet in fleets.iter_mut() {
        if fleet.state == FleetState::Orbiting {
            if credits.amount >= fleet.daily_upkeep {
                credits.amount -= fleet.daily_upkeep;
            } else {
                // Bankruptcy! Hostile takeover.
                fleet.state = FleetState::Occupying;

                // Seize all player buildings (simplified logic)
                for mut faction in buildings.iter_mut() {
                    if *faction == Faction::Player {
                        *faction = Faction::Mercenary;
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Seizing *all* buildings is too blunt. The system should identify the "most profitable industrial zones" by sorting buildings by a `Value` or `PowerGeneration` component and only seizing the top tier to cover the debt.
- **Integration**: When `FleetState::Occupying` triggers, it should fire a `ChronicleEvent` and change the UI state to reflect the loss of control over the seized zones.
- **Gameplay**: The player needs a way to pay off the debt and reclaim the buildings, perhaps by accumulating a lump sum or through an armed uprising.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Active MercenaryFleets deduct credits on their periodic update.
- [ ] Reaching 0 credits while a fleet is orbiting triggers a takeover, changing building factions to Mercenary.

## 7. Technical Guidance
- Buildings occupied by `Faction::Mercenary` should immediately sever their connection to the player's Layer 1 resource network (e.g., stop providing power to player zones).

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
