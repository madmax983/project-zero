# 585: Biocompatibility

## 1. Overview
**Layer:** 1
**Fantasy:** The planet's biology rejects you.
**Mechanic:** Pops have a "Biocompatibility" rating with the local flora/atmosphere. Low rating = sickness/slower work in unsealed areas. Can be improved via gene-modding or drugs.

## 2. Dependencies
- Layer 1 Core Needs (Health)
- Layer 1 Work Execution (Speed modifier)
- Layer 1 Environment (Atmosphere/Flora tags)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_biocompatibility_initialization() {
        let mut app = App::new();
        // Setup world with base incompatibility
        app.insert_resource(PlanetaryFlora { toxicity: 0.8 });

        let pop = app.world_mut().spawn(Pop::default()).id();
        app.add_systems(Update, initialize_biocompatibility_system);
        app.update();

        let compat = app.world().get::<Biocompatibility>(pop).unwrap();
        assert_eq!(compat.rating, 0.2); // 1.0 - 0.8 toxicity
    }

    #[test]
    fn test_low_compatibility_causes_sickness_tick() {
        let mut app = App::new();
        let pop = app.world_mut().spawn((
            Pop::default(),
            Biocompatibility { rating: 0.1 },
            Health { current: 100.0, max: 100.0 }
        )).id();

        app.add_systems(Update, environmental_exposure_system);
        app.update();

        let health = app.world().get::<Health>(pop).unwrap();
        assert!(health.current < 100.0, "Health should degrade due to low compatibility");
    }

    #[test]
    fn test_low_compatibility_reduces_work_speed() {
        let mut app = App::new();
        let pop = app.world_mut().spawn((
            Pop::default(),
            Biocompatibility { rating: 0.5 },
            WorkModifier { speed_multiplier: 1.0 }
        )).id();

        app.add_systems(Update, apply_biocompatibility_work_penalty_system);
        app.update();

        let modifier = app.world().get::<WorkModifier>(pop).unwrap();
        assert_eq!(modifier.speed_multiplier, 0.5, "Work speed should scale with compatibility");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
#[derive(Component)]
pub struct Biocompatibility {
    pub rating: f32, // 0.0 to 1.0
}

#[derive(Resource)]
pub struct PlanetaryFlora {
    pub toxicity: f32,
}

pub fn initialize_biocompatibility_system(
    mut commands: Commands,
    query: Query<Entity, Added<Pop>>,
    flora: Res<PlanetaryFlora>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(Biocompatibility {
            rating: 1.0 - flora.toxicity,
        });
    }
}

pub fn environmental_exposure_system(
    mut query: Query<(&Biocompatibility, &mut Health)>,
) {
    for (compat, mut health) in query.iter_mut() {
        if compat.rating < 0.3 {
            health.current -= 1.0; // Minimal implementation tick
        }
    }
}

pub fn apply_biocompatibility_work_penalty_system(
    mut query: Query<(&Biocompatibility, &mut WorkModifier)>,
) {
    for (compat, mut modif) in query.iter_mut() {
        modif.speed_multiplier = compat.rating;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring:** The hardcoded `0.3` threshold for sickness and `-1.0` health drain should be extracted into configurable `BiocompatibilityRules` resource.
- **Code Smells:** `apply_biocompatibility_work_penalty_system` directly overwrites `speed_multiplier` without considering other modifiers (like hunger or morale). It needs a multiplicative modifier system.
- **Performance:** Iterating over all Pops every frame for environmental exposure is costly. Consider bucketing or a stepped timer.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops spawn with correctly calculated Biocompatibility based on planetary traits.
- [ ] Pops with low Biocompatibility lose health over time when exposed.
- [ ] Work speed is proportionally reduced by low Biocompatibility.

## 7. Technical Guidance
- Ensure `Biocompatibility` only drops health if the Pop is *unsealed*. If they are inside a pressurized habitat (Layer 1 Environment), skip the `environmental_exposure_system` tick.
- Introduce an `AdaptationDrug` item that provides a temporary buff to `rating`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
