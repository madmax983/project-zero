# Biometric Identity Theft (Spec 470)

## Overview
When colony security is low, "Criminal" Pops can steal the biometric signatures of high-status Pops (like the Governor or Chief Engineer). The Criminal can then bypass locked doors or access restricted resources (like private stashes or weapons). This creates a tension between high security (which slows movement and costs admin) versus the risk of catastrophic internal sabotage and theft.

## Dependencies
- `142` Biometric Lockouts (Implemented)
- `103` Private Stashes (Implemented)
- `072` Justice System (Implemented)

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use super::*;

    #[test]
    fn test_criminal_steals_biometric_signature() {
        let mut world = World::new();

        // Arrange
        let victim = world.spawn((
            PopBundle::default(),
            BiometricSignature::new("governor_sig"),
            Job::Governor,
        )).id();

        let criminal = world.spawn((
            PopBundle::default(),
            Trait::Criminal,
        )).id();

        world.insert_resource(ColonySecurity { level: 0.1 }); // Low security

        // Act
        world.run_system_once(biometric_theft_system).unwrap();

        // Assert
        let criminal_identity = world.get::<StolenIdentity>(criminal).unwrap();
        assert_eq!(criminal_identity.target, victim);
        assert_eq!(criminal_identity.signature, "governor_sig");
    }

    #[test]
    fn test_high_security_prevents_theft() {
        let mut world = World::new();

        let victim = world.spawn((
            PopBundle::default(),
            BiometricSignature::new("engineer_sig"),
            Job::ChiefEngineer,
        )).id();

        let criminal = world.spawn((
            PopBundle::default(),
            Trait::Criminal,
        )).id();

        world.insert_resource(ColonySecurity { level: 0.9 }); // High security

        world.run_system_once(biometric_theft_system).unwrap();

        assert!(world.get::<StolenIdentity>(criminal).is_none());
    }

    #[test]
    fn test_criminal_accesses_restricted_resource() {
        let mut world = World::new();

        let victim = world.spawn((
            PopBundle::default(),
            BiometricSignature::new("vip_sig"),
        )).id();

        let criminal = world.spawn((
            PopBundle::default(),
            Trait::Criminal,
            StolenIdentity { target: victim, signature: "vip_sig".to_string() },
        )).id();

        let vault = world.spawn((
            Storage::new(100),
            BiometricLock::new(vec!["vip_sig".to_string()]),
        )).id();

        // Act
        world.run_system_once(unauthorized_access_system).unwrap();

        // Assert
        let theft_event = world.resource::<Events<TheftEvent>>().get_reader().read(&world.resource::<Events<TheftEvent>>()).next().unwrap();
        assert_eq!(theft_event.thief, criminal);
        assert_eq!(theft_event.target, vault);
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Component)]
pub struct StolenIdentity {
    pub target: Entity,
    pub signature: String,
}

pub fn biometric_theft_system(
    mut commands: Commands,
    security: Res<ColonySecurity>,
    criminals: Query<Entity, (With<Trait::Criminal>, Without<StolenIdentity>)>,
    victims: Query<(Entity, &BiometricSignature, &Job)>,
) {
    let mut rng = rand::thread_rng();

    // Base chance modified by security level
    let theft_chance = 0.05 * (1.0 - security.level);

    for criminal_entity in criminals.iter() {
        if rng.gen::<f32>() < theft_chance {
            // Find a high-value target (simplified: just grab the first one we find)
            if let Some((victim_entity, signature, _)) = victims.iter().find(|(_, _, job)| **job == Job::Governor || **job == Job::ChiefEngineer) {
                commands.entity(criminal_entity).insert(StolenIdentity {
                    target: victim_entity,
                    signature: signature.id.clone(),
                });
            }
        }
    }
}

pub fn unauthorized_access_system(
    mut events: EventWriter<TheftEvent>,
    criminals: Query<(Entity, &StolenIdentity)>,
    vaults: Query<(Entity, &BiometricLock)>,
) {
    for (criminal_entity, identity) in criminals.iter() {
        for (vault_entity, lock) in vaults.iter() {
            if lock.allowed_signatures.contains(&identity.signature) {
                events.send(TheftEvent {
                    thief: criminal_entity,
                    target: vault_entity,
                });
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design
- **Target Selection:** The current victim selection is naive. Criminals should prefer targets with access to specific resources they need.
- **Discovery Mechanic:** Add a system where `StolenIdentity` can be discovered during random security sweeps, triggering justice events.
- **Event Logging:** Theft should trigger a `ChronicleEvent` to create a permanent record of the "witch hunt" narrative.
- **Security Cost:** Ensure `ColonySecurity` resource clearly ties into colony upkeep or admin costs to emphasize the tension.

## Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Criminal pops can acquire `StolenIdentity` when security is low.
- [ ] Pops with `StolenIdentity` can bypass `BiometricLock`s they wouldn't normally have access to.

## Technical Guidance
- `StolenIdentity` should probably have a duration or decay over time as biometric signatures are cycled.
- Make sure to integrate with the existing `Justice System` (Spec 072) so criminals can be caught and punished.
- You'll likely need to extend `ActionType` in `src/layer1/utility_ai.rs` to include a `Theft` or `Infiltrate` action, rather than doing it purely probabilistically in `biometric_theft_system`.

## Questions
*Builder: add questions here if spec is unclear.*
