# 173: Predictive Policing

## Overview

"The best way to solve a crime is to arrest the criminal before they commit it."

This feature introduces a **Predictive Policing** mechanic where the colony's security forces can identify Pops at high risk of mental breakdowns or criminal behavior (e.g., Vandalism, Violence) *before* the event occurs. High-risk Pops are marked as `Suspect` and can be preemptively arrested by Wardens (`ActionType::PreCrimeArrest`).

Arresting Suspects prevents the breakdown/crime but may cause "Tyranny" (morale penalties) or "Unrest" if the prediction was wrong or if the population values Liberty.

## Dependencies

- `072` — Justice System (Warden role, Jail zones)
- `127` — Stress Breakdowns (StressTracker, Breakdown types)
- `011` — Tech Tree Backend (Unlockable tech)
- `004` — Building System (Security Station / Algo-Hub)

## RED Phase: Tests First

Write these tests in `src/layer1/predictive_policing_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::stress::StressTracker;
    use crate::layer1::traits::{Trait, Traits};
    use crate::layer1::justice::{Wanted, Inmate}; // Existing components from 072
    use crate::layer1::predictive_policing::{
        Suspect, PredictiveModel, PredictionConfig,
        check_prediction_system, evaluate_pre_crime_arrest
    };
    use crate::layer1::map::GridPosition;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(PredictionConfig {
            threshold: 0.8, // 80% probability required
            enabled: true,
        });
        world
    }

    // 1. Prediction Logic
    #[test]
    fn test_high_stress_volatile_pop_becomes_suspect() {
        let mut world = setup_world();
        let mut schedule = Schedule::default();
        schedule.add_systems(check_prediction_system);

        // Volatile pop with high stress ticks
        let mut traits = std::collections::HashSet::new();
        traits.insert(Trait::Volatile); // Assumes Trait::Volatile exists or is added

        let pop = world.spawn((
            Pop::default(),
            StressTracker { ticks_at_low_morale: 80 }, // Near breakdown (threshold is 100)
            Traits(traits),
            // No Suspect component yet
        )).id();

        schedule.run(&mut world);

        // Should be marked Suspect
        let suspect = world.get::<Suspect>(pop).unwrap();
        assert!(suspect.probability > 0.8);
        assert_eq!(suspect.predicted_crime, "Breakdown: Violence");
    }

    #[test]
    fn test_low_stress_pop_is_safe() {
        let mut world = setup_world();
        let mut schedule = Schedule::default();
        schedule.add_systems(check_prediction_system);

        let pop = world.spawn((
            Pop::default(),
            StressTracker { ticks_at_low_morale: 10 },
            Traits(std::collections::HashSet::new()),
        )).id();

        schedule.run(&mut world);

        assert!(world.get::<Suspect>(pop).is_none());
    }

    // 2. Warden Evaluation
    #[test]
    fn test_warden_targets_suspect() {
        let mut world = setup_world();

        let suspect = world.spawn((
            Pop::default(),
            Suspect { probability: 0.9, predicted_crime: "Arson".to_string() },
            GridPosition { x: 5, y: 5 },
        )).id();

        let warden_pos = GridPosition { x: 0, y: 0 };

        // Evaluate action
        let result = evaluate_pre_crime_arrest(&world, &warden_pos);

        assert!(result.is_some());
        let (score, target) = result.unwrap();
        assert!(score > 0.0);
        assert_eq!(target, suspect);
    }

    // 3. Arrest Execution
    #[test]
    fn test_pre_crime_arrest_converts_to_inmate() {
        let mut world = setup_world();

        // Register required resources/systems from 072 if needed,
        // or mock the arrest execution function for unit testing logic.
        // Here we test a specific pre-crime arrest handler.

        let suspect = world.spawn((
            Pop::default(),
            Suspect { probability: 0.95, predicted_crime: "Murder".to_string() },
            GridPosition { x: 1, y: 1 },
        )).id();

        let warden = world.spawn((
            Pop::default(),
            GridPosition { x: 0, y: 0 },
        )).id();

        // Execute arrest
        crate::layer1::predictive_policing::execute_pre_crime_arrest(&mut world, warden, suspect);

        // Should be Inmate (Protective Custody)
        let inmate = world.get::<Inmate>(suspect).unwrap();
        assert!(inmate.sentence_ticks > 0);
        // Should NOT be Wanted (they haven't done it yet)
        assert!(world.get::<Wanted>(suspect).is_none());
        // Suspect marker removed
        assert!(world.get::<Suspect>(suspect).is_none());
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components (`src/layer1/predictive_policing.rs`)

```rust
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct Suspect {
    pub probability: f32, // 0.0 to 1.0
    pub predicted_crime: String,
}

#[derive(Resource, Default)]
pub struct PredictionConfig {
    pub threshold: f32,
    pub enabled: bool,
}

// Marker for the "Algo-Hub" building that enables this
#[derive(Component)]
pub struct PredictiveModel;
```

### 2. Systems

```rust
use crate::layer1::stress::StressTracker;
use crate::layer1::traits::{Trait, Traits};
use crate::layer1::justice::Inmate;
use crate::layer1::map::GridPosition;

pub fn check_prediction_system(
    mut commands: Commands,
    query: Query<(Entity, &StressTracker, Option<&Traits>), Without<Suspect>>,
    config: Res<PredictionConfig>,
) {
    if !config.enabled { return; }

    for (entity, tracker, traits) in query.iter() {
        let mut risk = 0.0;

        // Stress factor
        if tracker.ticks_at_low_morale > 50 {
            risk += 0.5;
        }

        // Trait factor
        if let Some(t) = traits {
            if t.0.contains(&Trait::Volatile) { // Assuming Trait::Volatile
                risk += 0.3;
            }
            if t.0.contains(&Trait::Pyromaniac) {
                risk += 0.4;
            }
        }

        if risk >= config.threshold {
            commands.entity(entity).insert(Suspect {
                probability: risk.min(1.0),
                predicted_crime: "Predicted Breakdown".to_string(),
            });
        }
    }
}

pub fn evaluate_pre_crime_arrest(
    world: &World,
    warden_pos: &GridPosition,
) -> Option<(f32, Entity)> {
    // Naive search for nearest Suspect
    // In real implementation, check for 'Warden' job assignment or zone
    let mut best_target = None;
    let mut min_dist = i32::MAX;

    let mut query = world.query::<(Entity, &GridPosition, &Suspect)>();
    for (entity, pos, _suspect) in query.iter(world) {
        let dist = crate::layer1::utility_types::manhattan_distance(warden_pos, pos);
        if dist < min_dist {
            min_dist = dist;
            best_target = Some(entity);
        }
    }

    if let Some(target) = best_target {
        // Score slightly lower than actual crime arrest (0.8 vs 0.9)
        return Some((0.7, target));
    }
    None
}

pub fn execute_pre_crime_arrest(
    world: &mut World,
    _warden: Entity,
    target: Entity,
) {
    // Remove Suspect
    world.entity_mut(target).remove::<Suspect>();

    // Add Inmate with "Protective Custody" sentence (shorter than crime)
    world.entity_mut(target).insert(Inmate {
        sentence_ticks: 500, // Short sentence to cool down
    });

    // Reset stress?
    if let Some(mut tracker) = world.get_mut::<StressTracker>(target) {
        tracker.ticks_at_low_morale = 0; // The arrest "resets" the breakdown buildup (via shock/containment)
    }

    // Teleport to jail (Reuse logic from 072 via helper or duplicate for MVP)
    // ...
}
```

### 3. Update ActionType

Update `src/layer1/utility_types.rs`:
Add `PreCrimeArrest` variant. Update `COUNT` to **29**.

## REFACTOR Phase: Quality & Design

- **Algo-Hub Requirement**: The `check_prediction_system` should only run if a `PredictiveModel` building exists and is powered.
- **False Positives**: Introduce a chance that the prediction is wrong. If an innocent Suspect is arrested, they gain a "Resentful" trait or "Unjustly Imprisoned" memory (Spec 036).
- **Ui Integration**: Show Suspect probability in the Inspector window.
- **Policy**: Add a Colony Edict (`054`) to toggle "Pre-Crime" on/off globally.

## Acceptance Criteria

- [ ] `Suspect` component added to Pops exceeding risk threshold.
- [ ] `ActionType::PreCrimeArrest` added (Count 29).
- [ ] Wardens successfully arrest Suspects.
- [ ] Arrested Suspects become Inmates and have their Stress reset.
- [ ] Tests pass.

## Technical Guidance

- **ActionType Count**: You must increment `ActionType::COUNT` from 28 to 29 in `src/layer1/utility_types.rs`.
- **GPU Buffers**: Update `GpuPopInput` in `src/gpu/buffers.rs` to align with 29 actions.
    - Previous: 28 u32s.
    - New: 29 u32s.
    - Padding calculation:
        - `success_count`: 29 * 4 = 116 bytes.
        - `attempt_count`: 29 * 4 = 116 bytes.
        - Total arrays: 232 bytes.
        - Plus other fields...
        - Ensure 16-byte alignment is maintained (pad to nearest 16).
- **Traits**: If `Trait::Volatile` does not exist, add it to `src/layer1/traits.rs`.

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
