# 539 - Penal Contracts

## 1. Overview
Accept contracts from Layer 3 civilizations to house "State Prisoners". You get paid per head per day. You must keep them alive and contained. If they escape or die, you face heavy penalties/hostility.

## 2. Dependencies
- 003 Population Basics
- 072 Justice System
- 039 Trade System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_penal_contract_payment() {
        // Arrange
        let mut world = World::new();
        let colony = world.spawn(ColonyFunds(0)).id();
        let contract = world.spawn((PenalContract { payment_per_day: 10, prisoner_count: 5 }, ContractTimer(1))).id();

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(process_penal_contracts_system);
        schedule.run(&mut world);

        // Assert
        let funds = world.get::<ColonyFunds>(colony).unwrap();
        assert_eq!(funds.0, 50, "Colony should receive 50 credits (5 prisoners * 10/day)");
    }

    #[test]
    fn test_prisoner_death_penalty() {
        // Arrange
        let mut world = World::new();
        let faction = world.spawn(FactionRelation(100)).id();
        let prisoner = world.spawn((Pop, PrisonerOf(faction), Health(0))).id();

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(check_prisoner_status_system);
        schedule.run(&mut world);

        // Assert
        let relation = world.get::<FactionRelation>(faction).unwrap();
        assert!(relation.0 < 100, "Faction relation should drop if prisoner dies");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct ColonyFunds(pub u32);

#[derive(Component)]
pub struct PenalContract {
    pub payment_per_day: u32,
    pub prisoner_count: u32,
}

#[derive(Component)]
pub struct ContractTimer(pub u32);

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct PrisonerOf(pub Entity);

#[derive(Component)]
pub struct FactionRelation(pub i32);

#[derive(Component)]
pub struct Health(pub i32);

pub fn process_penal_contracts_system(
    mut routes: Query<(&PenalContract, &mut ContractTimer)>,
    mut colonies: Query<&mut ColonyFunds>,
) {
    for (contract, mut timer) in routes.iter_mut() {
        if timer.0 > 0 {
            timer.0 -= 1;
        }
        if timer.0 == 0 {
            if let Ok(mut funds) = colonies.get_single_mut() {
                funds.0 += contract.payment_per_day * contract.prisoner_count;
            }
            timer.0 = 1;
        }
    }
}

pub fn check_prisoner_status_system(
    prisoners: Query<(&PrisonerOf, &Health), With<Pop>>,
    mut factions: Query<&mut FactionRelation>,
) {
    for (prisoner_of, health) in prisoners.iter() {
        if health.0 <= 0 {
            if let Ok(mut relation) = factions.get_mut(prisoner_of.0) {
                relation.0 -= 50; // Heavy penalty
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create actual prisoner entities arriving via shuttle.
- Separate contract validation from daily processing.
- Handle escapes as well as deaths.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Payments are made daily.
- [ ] Death/escape applies faction penalties.

## 7. Technical Guidance
- Prisoners should have a special trait or marker making them distinct from normal citizens.
- They should not be allowed to perform regular jobs outside of designated Penal Zones.

## 8. Questions
*Builder: add questions here if spec is unclear.*
