# Specification: The Kinetic Heist

## 1. Overview
The Kinetic Heist feature enables pirates or hostile factions to fire a "Harvester Meteor" from Layer 2 into a Layer 1 colony. This indestructible kinetic projectile crashes into the colony, absorbs valuable refined materials from the impact zone, and then launches itself back into orbit to steal the resources unless dismantled within a time limit.

## 2. Dependencies
- Layer 2 -> Layer 1 Interaction (Bombardment/Projectiles)
- Layer 1 Resources and Inventories
- Layer 1 Combat/Dismantling mechanics

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use super::*;

    #[derive(Component)]
    struct HarvesterMeteor {
        stolen_amount: u32,
        launch_timer: f32,
    }

    #[derive(Component)]
    struct ResourceVault {
        alloys: u32,
    }

    #[derive(Component)]
    struct ImpactZone;

    // The system to test
    fn harvester_meteor_absorption_system(
        mut meteors: Query<&mut HarvesterMeteor>,
        mut vaults: Query<(&mut ResourceVault, &ImpactZone)>,
    ) {
        // Implementation goes here
    }

    fn spawn_test_world() -> World {
        World::new()
    }

    #[test]
    fn test_meteor_absorbs_resources() {
        let mut world = spawn_test_world();

        let meteor_entity = world.spawn().id();
        world.entity_mut(meteor_entity).insert(HarvesterMeteor {
            stolen_amount: 0,
            launch_timer: 180.0,
        });

        let vault_entity = world.spawn().id();
        world.entity_mut(vault_entity).insert((
            ResourceVault { alloys: 500 },
            ImpactZone,
        ));

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(harvester_meteor_absorption_system);
        schedule.run(&mut world);

        // Assert
        let vault = world.get::<ResourceVault>(vault_entity).unwrap();
        let meteor = world.get::<HarvesterMeteor>(meteor_entity).unwrap();

        assert_eq!(vault.alloys, 0, "Vault should be drained by the meteor");
        assert_eq!(meteor.stolen_amount, 500, "Meteor should absorb the vault's resources");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
fn harvester_meteor_absorption_system(
    mut meteors: Query<&mut HarvesterMeteor>,
    mut vaults: Query<(&mut ResourceVault, &ImpactZone)>,
) {
    for mut meteor in meteors.iter_mut() {
        for (mut vault, _zone) in vaults.iter_mut() {
            let amount = vault.alloys;
            vault.alloys = 0;
            meteor.stolen_amount += amount;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Refactoring:** Ensure the `ImpactZone` correctly maps to the exact coordinates of the `HarvesterMeteor`. Currently, the minimal implementation blindly drains all vaults marked as `ImpactZone`.
- **Code Smells:** Resource types shouldn't be hardcoded to `alloys`. Use the existing `Inventory` or `ResourceMap` components to drain all high-tier materials.
- **Performance:** Limit absorption to run only on the initial tick of impact, not continuously, to prevent redundant checks.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new code.
- [ ] Harvester Meteors successfully drain resources upon impact.

## 7. Technical Guidance
- Implement in `layer1/hazards/meteor.rs` or `layer2/combat.rs`.
- Ensure a dismantling mechanic exists so Pops can target the Meteor as an enemy entity to stop the launch timer.

## 8. Questions
*Builder: add questions here if spec is unclear.*
