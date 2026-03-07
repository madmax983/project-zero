# 421: The Pirate Emissary

## 1. Overview
**Layer:** Cross-layer (2 & 1)
**Fantasy:** Hiring the wolves to guard the sheep.
**Mechanic:** You can hire a powerful Layer 2 Pirate Fleet to act as "Privateers" for your system. They defend your trade routes and attack rivals, but they demand a "Tribute" of resources or Pops. If you fail to pay, they turn their guns on your capital.
**Emergence:** The Privateers are so effective that your economy booms. But their tribute demands scale with your wealth. Eventually, you realize you're just a farm for the pirates, and you have to build a secret navy to destroy your own protectors.
**Tension:** Unmatched early security vs. A ticking time bomb of extortion.

## 2. Dependencies
- Layer 2 Fleet system
- Trade and tribute economic system
- Faction relationships system

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_privateer_demands_tribute_and_turns_hostile_if_unpaid() {
        // Arrange
        let mut world = World::new();
        let mut app = App::new();
        // Setup player colony resources
        world.insert_resource(PlayerTreasury { credits: 50.0 });

        let pirate_fleet = world.spawn((
            PrivateerContract { tribute_demand: 100.0, paid: false },
            Faction::Pirate,
            Hostility(0.0), // Non-hostile initially
        )).id();

        // Act
        app.add_systems(Update, privateer_tribute_system);
        app.update();

        // Assert
        let hostility = world.get::<Hostility>(pirate_fleet).unwrap().0;
        assert!(hostility > 0.0, "Pirate fleet should become hostile when tribute is unpaid");
    }

    #[test]
    fn test_privateer_protects_player_trade_when_paid() {
        // Arrange
        let mut world = World::new();
        let mut app = App::new();
        // Setup player colony resources
        world.insert_resource(PlayerTreasury { credits: 200.0 });

        let pirate_fleet = world.spawn((
            PrivateerContract { tribute_demand: 100.0, paid: true },
            Faction::Pirate,
            ProtectionBuff::default(),
        )).id();

        // Act
        app.add_systems(Update, privateer_protection_system);
        app.update();

        // Assert
        let buff = world.get::<ProtectionBuff>(pirate_fleet).unwrap();
        assert!(buff.active, "Pirate fleet should provide protection buff when tribute is paid");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct PrivateerContract {
    pub tribute_demand: f32,
    pub paid: bool,
}

#[derive(Component, Default)]
pub struct ProtectionBuff {
    pub active: bool,
}

#[derive(Component)]
pub struct Hostility(pub f32);

#[derive(Component, PartialEq)]
pub enum Faction {
    Pirate,
    Player,
}

#[derive(Resource)]
pub struct PlayerTreasury {
    pub credits: f32,
}

pub fn privateer_tribute_system(
    mut treasury: ResMut<PlayerTreasury>,
    mut pirate_query: Query<(&mut PrivateerContract, &mut Hostility)>,
) {
    for (mut contract, mut hostility) in pirate_query.iter_mut() {
        if treasury.credits >= contract.tribute_demand {
            treasury.credits -= contract.tribute_demand;
            contract.paid = true;
        } else {
            hostility.0 = 100.0; // Turn hostile
            contract.paid = false;
        }
    }
}

pub fn privateer_protection_system(
    mut pirate_query: Query<(&PrivateerContract, &mut ProtectionBuff)>,
) {
    for (contract, mut protection) in pirate_query.iter_mut() {
        protection.active = contract.paid;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Extract the tribute interval and amount calculation to a robust event system or resource.
- Introduce dynamic scaling of tribute demands based on player wealth/production.
- Combine the logic into a more cohesive contract state machine (e.g., `ContractState::Pending`, `ContractState::Active`, `ContractState::Breached`).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pirate Emissary system functions to demand tribute and turn hostile if unpaid.

## 7. Technical Guidance
- Integrate with Layer 2 combat system to ensure `Hostility` actually triggers attacks on the player's assets.
- Make the tribute deduction a transaction that the player can explicitly reject or automate.

## 8. Questions
*Builder: add questions here if spec is unclear.*
