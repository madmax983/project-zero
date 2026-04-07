# Specification: 863 Corporate Foreclosure

## 1. Overview
**Layer:** Cross-layer (1 & 3)
**Fantasy:** You didn't read the terms of service.
**Mechanic:** If you are in debt to a Layer 3 faction (Loans), they can "Foreclose" on specific Layer 1 buildings. They take ownership (you lose control), and the output goes to them. You must pay to buy them back.

## 2. Dependencies
- Layer 1 Buildings (`src/layer1/building.rs`)
- Layer 3 Diplomacy/Factions (`src/layer3/diplomacy.rs`)
- Economy System (Loans & Debt)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_building_foreclosure_transfers_ownership() {
        let mut app = App::new();
        app.add_plugins(ForeclosurePlugin);

        let faction = app.world_mut().spawn(FactionNode).id();
        let building = app.world_mut().spawn((
            Building,
            Ownership::Player,
            OutputResource { amount: 10, target_faction: None },
        )).id();

        app.world_mut().send_event(ForecloseEvent {
            target_building: building,
            new_owner: faction,
        });
        app.update();

        let new_ownership = app.world().get::<Ownership>(building).unwrap();
        assert_eq!(*new_ownership, Ownership::Faction(faction), "Building ownership should transfer to faction");

        let output = app.world().get::<OutputResource>(building).unwrap();
        assert_eq!(output.target_faction, Some(faction), "Building output should be redirected to new owner");
    }

    #[test]
    fn test_buyback_restores_ownership() {
        let mut app = App::new();
        app.add_plugins(ForeclosurePlugin);

        let faction = app.world_mut().spawn(FactionNode).id();
        let building = app.world_mut().spawn((
            Building,
            Ownership::Faction(faction),
            OutputResource { amount: 10, target_faction: Some(faction) },
        )).id();

        app.world_mut().send_event(BuybackEvent {
            target_building: building,
        });
        app.update();

        let ownership = app.world().get::<Ownership>(building).unwrap();
        assert_eq!(*ownership, Ownership::Player, "Building ownership should return to player");

        let output = app.world().get::<OutputResource>(building).unwrap();
        assert_eq!(output.target_faction, None, "Building output should not be redirected");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct FactionNode;

#[derive(Component)]
pub struct Building;

#[derive(Component, PartialEq, Debug)]
pub enum Ownership {
    Player,
    Faction(Entity),
}

#[derive(Component)]
pub struct OutputResource {
    pub amount: i32,
    pub target_faction: Option<Entity>,
}

#[derive(Event)]
pub struct ForecloseEvent {
    pub target_building: Entity,
    pub new_owner: Entity,
}

#[derive(Event)]
pub struct BuybackEvent {
    pub target_building: Entity,
}

pub struct ForeclosurePlugin;

impl Plugin for ForeclosurePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ForecloseEvent>()
           .add_event::<BuybackEvent>()
           .add_systems(Update, (
               handle_foreclosure_system,
               handle_buyback_system,
           ));
    }
}

fn handle_foreclosure_system(
    mut events: EventReader<ForecloseEvent>,
    mut buildings: Query<(&mut Ownership, &mut OutputResource)>,
) {
    for event in events.read() {
        if let Ok((mut ownership, mut output)) = buildings.get_mut(event.target_building) {
            *ownership = Ownership::Faction(event.new_owner);
            output.target_faction = Some(event.new_owner);
        }
    }
}

fn handle_buyback_system(
    mut events: EventReader<BuybackEvent>,
    mut buildings: Query<(&mut Ownership, &mut OutputResource)>,
) {
    for event in events.read() {
        if let Ok((mut ownership, mut output)) = buildings.get_mut(event.target_building) {
            *ownership = Ownership::Player;
            output.target_faction = None;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- `ForecloseEvent` currently assumes the building is valid and player-owned. Add validation checks before applying ownership transfer.
- Ensure that UI systems correctly read `Ownership` to visually lock or flag foreclosed buildings.
- Link the `BuybackEvent` directly to the `Economy` system so the player's credits are actually deducted.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the module.
- [ ] Sending a `ForecloseEvent` updates the building's `Ownership` and `OutputResource.target_faction`.
- [ ] Sending a `BuybackEvent` reverts the building's `Ownership` to Player.

## 7. Technical Guidance
- The actual event firing logic for `ForecloseEvent` should be driven by the faction AI evaluating debt limits in Layer 3.
- When output is redirected, it means that during the `work_execution_system`, the produced goods bypass local colony storage and are sent to a virtual export queue or directly credited to the Faction's ledger.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
