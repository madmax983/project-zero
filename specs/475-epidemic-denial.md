# 475: Epidemic Denial

## 1. Overview
A plague is ravaging the colony, but half the population thinks it's a hoax by the governor. During an outbreak, Pops with "Rebellious" or "Paranoid" traits have a chance to gain the "Denial" condition. They refuse to visit the hospital, take medicine, or respect quarantine zones, actively spreading the disease and reducing the overall health rating. This forces the player to decide between violently enforcing quarantine (causing massive unrest) or trying to reason with deniers while the infection spreads.

## 2. Dependencies
- `034` Pop Health and Damage
- `084` Pop Traits
- `068` Pop Factions

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    // Mock components based on dependencies
    // use scale_core::layer1::pop::{Trait, PopHealth, Infected};
    // use scale_core::layer1::medical::{MedicalCareStatus, QuarantineZone};

    #[derive(Component, PartialEq, Eq, Clone, Copy)]
    enum Trait {
        Rebellious,
        Paranoid,
        Gullible,
    }

    #[derive(Component)]
    struct PopHealth {
        pub value: f32,
    }

    #[derive(Component)]
    struct Infected {
        pub disease_id: String,
        pub severity: f32,
    }

    #[derive(Component)]
    struct EpidemicDenial;

    #[derive(Component)]
    struct MedicalCareStatus {
        pub receiving_care: bool,
    }

    #[derive(Component)]
    struct TargetQuarantineZone {
        pub target: Entity,
    }

    #[derive(Component)]
    struct QuarantineZone;

    fn check_for_denial_system(
        mut commands: Commands,
        query: Query<(Entity, &Infected, &Trait), Without<EpidemicDenial>>,
    ) {
        for (entity, _infected, pop_trait) in query.iter() {
            if *pop_trait == Trait::Rebellious || *pop_trait == Trait::Paranoid {
                commands.entity(entity).insert(EpidemicDenial);
            }
        }
    }

    fn apply_denial_effects_system(
        mut commands: Commands,
        mut query: Query<(Entity, &EpidemicDenial, Option<&mut MedicalCareStatus>, Option<&TargetQuarantineZone>)>,
    ) {
        for (entity, _denial, mut med_care, target_zone) in query.iter_mut() {
            if let Some(mut care) = med_care {
                care.receiving_care = false;
            }
            if target_zone.is_some() {
                commands.entity(entity).remove::<TargetQuarantineZone>();
            }
        }
    }

    #[test]
    fn test_paranoid_pop_gains_denial() {
        let mut app = App::new();

        let pop = app.world_mut().spawn((
            PopHealth { value: 100.0 },
            Infected { disease_id: "SpaceRot".to_string(), severity: 10.0 },
            Trait::Paranoid,
        )).id();

        app.add_systems(Update, check_for_denial_system);
        app.update();

        assert!(app.world().entity(pop).contains::<EpidemicDenial>(), "Paranoid infected pop should gain EpidemicDenial");
    }

    #[test]
    fn test_denier_refuses_care_and_quarantine() {
        let mut app = App::new();
        let quarantine_zone = app.world_mut().spawn(QuarantineZone).id();

        let pop = app.world_mut().spawn((
            PopHealth { value: 100.0 },
            Infected { disease_id: "SpaceRot".to_string(), severity: 10.0 },
            EpidemicDenial,
            MedicalCareStatus { receiving_care: true },
            TargetQuarantineZone { target: quarantine_zone },
        )).id();

        app.add_systems(Update, apply_denial_effects_system);
        app.update();

        let care = app.world().get::<MedicalCareStatus>(pop).unwrap();
        assert!(!care.receiving_care, "Denier should refuse medical care");
        assert!(!app.world().entity(pop).contains::<TargetQuarantineZone>(), "Denier should ignore quarantine targets");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub enum Trait {
    Rebellious,
    Paranoid,
    Gullible,
    // ... other traits
}

#[derive(Component)]
pub struct Infected {
    pub disease_id: String,
    pub severity: f32,
}

#[derive(Component)]
pub struct EpidemicDenial;

#[derive(Component)]
pub struct MedicalCareStatus {
    pub receiving_care: bool,
}

#[derive(Component)]
pub struct TargetQuarantineZone {
    pub target: Entity,
}

pub fn check_for_denial_system(
    mut commands: Commands,
    query: Query<(Entity, &Infected, &Trait), Without<EpidemicDenial>>,
) {
    for (entity, _infected, pop_trait) in query.iter() {
        // In a full implementation, this should be a probability check rather than 100% guarantee
        if *pop_trait == Trait::Rebellious || *pop_trait == Trait::Paranoid {
            commands.entity(entity).insert(EpidemicDenial);
        }
    }
}

pub fn apply_denial_effects_system(
    mut commands: Commands,
    mut query: Query<(Entity, &EpidemicDenial, Option<&mut MedicalCareStatus>, Option<&TargetQuarantineZone>)>,
) {
    for (entity, _denial, mut med_care, target_zone) in query.iter_mut() {
        if let Some(mut care) = med_care {
            // Actively cancel care
            care.receiving_care = false;
        }
        if target_zone.is_some() {
            // Ignore quarantine orders
            commands.entity(entity).remove::<TargetQuarantineZone>();
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Probability Integration**: The gain of `EpidemicDenial` should not be 100% guaranteed. Introduce a dice roll based on the severity of the outbreak and the specific trait.
- **Contagion Mechanic**: Deniers should have a larger contagion radius or a higher chance of spreading the disease since they do not wear PPE or isolate.
- **Enforcement Edict**: Create an edict for "Violent Enforcement" of quarantine, which forcefully restrains Deniers at the cost of significantly dropping colony morale or sparking a riot.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new code.
- [ ] Infected pops with specific traits can gain `EpidemicDenial`.
- [ ] Pops with `EpidemicDenial` actively refuse/cancel `MedicalCareStatus` and `TargetQuarantineZone`.

## 7. Technical Guidance
- Ensure `apply_denial_effects_system` runs *after* any systems that assign medical care or quarantine targets in the schedule, effectively overriding those AI decisions.
- Hook into the notification system to alert the player when a denier breaks quarantine.

## Questions

*Builder: add questions here if spec is unclear. Architect will address.*
