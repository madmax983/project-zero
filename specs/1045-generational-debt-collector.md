# 1045: The Generational Debt Collector

## 1. Overview
A massive, heavily armed "Repo Fleet" from a Layer 3 Megabank arrives in your system. They claim the original seed-funding for your colony was a loan with compounding interest. They don't want money; they want physical collateral. They begin literally laser-carving out entire sectors of your Layer 1 colony to haul away. Fighting back flags you as "In Default" to the galactic market, collapsing trade. The player must choose between an unwinnable war or sacrificing sectors/Pops as collateral.

## 2. Dependencies
- Layer 1 Buildings and Sectors
- Layer 3 Diplomatic/Trade System
- Layer 2 Fleet Spawning

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use scale::layer3::trade::GalacticMarketStatus;

    #[test]
    fn test_repo_fleet_arrival_triggers_collection() {
        let mut app = App::new();
        app.add_event::<RepoFleetArrivalEvent>();
        app.add_systems(Update, handle_repo_fleet_arrival);

        app.world_mut().send_event(RepoFleetArrivalEvent { debt_amount: 1000000.0 });
        app.update();

        // Ensure a Collection Protocol is active
        assert!(app.world().get_resource::<ActiveCollectionProtocol>().is_some(), "Collection protocol should be active");
    }

    #[test]
    fn test_attacking_repo_fleet_causes_default() {
        let mut app = App::new();
        app.insert_resource(GalacticMarketStatus { in_default: false });
        app.add_event::<AttackRepoFleetEvent>();
        app.add_systems(Update, handle_repo_fleet_attack);

        app.world_mut().send_event(AttackRepoFleetEvent);
        app.update();

        // Verify the colony is in default
        let market_status = app.world().resource::<GalacticMarketStatus>();
        assert!(market_status.in_default, "Attacking the Repo Fleet should trigger market default");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Event)]
pub struct RepoFleetArrivalEvent {
    pub debt_amount: f32,
}

#[derive(Event)]
pub struct AttackRepoFleetEvent;

#[derive(Resource)]
pub struct ActiveCollectionProtocol {
    pub remaining_debt: f32,
}

// Mock of existing Layer 3 resource
pub mod scale {
    pub mod layer3 {
        pub mod trade {
            use bevy::prelude::Resource;
            #[derive(Resource)]
            pub struct GalacticMarketStatus {
                pub in_default: bool,
            }
        }
    }
}
use scale::layer3::trade::GalacticMarketStatus;

pub fn handle_repo_fleet_arrival(
    mut commands: Commands,
    mut events: EventReader<RepoFleetArrivalEvent>,
) {
    for event in events.read() {
        commands.insert_resource(ActiveCollectionProtocol {
            remaining_debt: event.debt_amount,
        });
    }
}

pub fn handle_repo_fleet_attack(
    mut events: EventReader<AttackRepoFleetEvent>,
    mut market_status: Option<ResMut<GalacticMarketStatus>>,
) {
    for _ in events.read() {
        if let Some(ref mut status) = market_status {
            status.in_default = true;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Sector Carving:** The minimal implementation only sets flags. The actual logic to select a grid sector on Layer 1 and delete the buildings/Pops inside it to satisfy the debt needs to be implemented.
- **Event integration:** `AttackRepoFleetEvent` should be triggered automatically when any player-owned Layer 2 military ship targets a ship belonging to the Repo Fleet faction.

## 6. Acceptance Criteria (Testable!)
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85%.
- [ ] Repo Fleet arrival begins the collection phase.
- [ ] Engaging the fleet immediately applies the `in_default` debuff to the Galactic Market.
- [ ] Sacrificing a building/sector reduces the remaining debt.

## 7. Technical Guidance
- Place the core logic in a new module in `src/layer3/events/`.
- The physical removal of Layer 1 tiles by a Layer 2 fleet is complex. Consider implementing a "Tractor Beam" or "Laser Cutter" action on the Layer 2 ships that targets Layer 1 coordinates over time.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
