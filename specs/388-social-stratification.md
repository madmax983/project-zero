# 388 - Social Stratification

## 1. Overview
**Layer:** 1
**Fantasy:** A society naturally divides itself. The "Clean Coats" in the labs vs. the "Dusty Boots" in the mines.
**Mechanic:** Jobs have hidden "Prestige" values. Pops working high-prestige jobs look down on low-prestige pops. Mixing housing/dining areas causes "Class Friction" (stress).
**Emergence:** You unwittingly create a ghetto for miners. When the life support fails, the scientists demand priority, sparking a civil war.
**Tension:** Mixed zoning (social friction) vs. Segregated zoning (efficient but creates factions).

## 2. Dependencies
- `Pop` and `Job` components
- `Needs` component (specifically tracking `stress` or `leisure`)
- Room/Zone assignments (e.g., dining areas, housing blocks)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::needs::Needs;
    use crate::layer1::jobs::{Job, JobType};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, apply_class_friction_system);
        app
    }

    #[test]
    fn test_class_friction_between_different_prestige() {
        let mut app = setup_app();

        // High prestige pop (Scientist)
        let high_pop = app.world_mut().spawn((
            Pop,
            Needs::default(),
            Job { job_type: JobType::Scientist, prestige: 10 },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        // Low prestige pop (Miner)
        let low_pop = app.world_mut().spawn((
            Pop,
            Needs::default(), // Leisure starts at 100.0 or similar
            Job { job_type: JobType::Miner, prestige: 1 },
            Transform::from_xyz(1.0, 0.0, 0.0), // Adjacent
        )).id();

        app.update();

        // Low prestige pop should lose leisure/gain stress due to proximity to high prestige pop
        let low_needs = app.world().get::<Needs>(low_pop).unwrap();
        assert!(low_needs.leisure < 100.0, "Leisure should drop due to friction");
    }

    #[test]
    fn test_no_class_friction_same_prestige() {
        let mut app = setup_app();

        let pop1 = app.world_mut().spawn((
            Pop,
            Needs::default(),
            Job { job_type: JobType::Miner, prestige: 1 },
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let pop2 = app.world_mut().spawn((
            Pop,
            Needs::default(),
            Job { job_type: JobType::Miner, prestige: 1 },
            Transform::from_xyz(1.0, 0.0, 0.0),
        )).id();

        app.update();

        let needs1 = app.world().get::<Needs>(pop1).unwrap();
        assert_eq!(needs1.leisure, 100.0, "Leisure should not drop");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::pop::Pop;
use crate::layer1::needs::Needs;
use crate::layer1::jobs::Job;

const FRICTION_RADIUS: f32 = 3.0;

pub fn apply_class_friction_system(
    mut query: Query<(&mut Needs, &Job, &Transform), With<Pop>>,
) {
    let mut combinations = query.iter_combinations_mut();
    while let Some([(mut needs1, job1, t1), (mut needs2, job2, t2)]) = combinations.fetch_next() {
        if t1.translation.distance(t2.translation) <= FRICTION_RADIUS {
            let prestige_diff = (job1.prestige as i32 - job2.prestige as i32).abs() as f32;
            if prestige_diff > 5.0 {
                // Apply a leisure drop (or stress increase)
                needs1.leisure -= prestige_diff * 0.1;
                needs2.leisure -= prestige_diff * 0.1;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Optimize using spatial hashing.
- Apply friction primarily in shared zones (Mess Hall, Barracks) using a `Room` query rather than raw proximity.
- Introduce an `Egalitarian` trait that negates this effect for specific Pops.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Proximity between disparate job prestige levels causes a measurable penalty to pop needs.

## 7. Technical Guidance
- Ensure the prestige drop acts upon `Needs.leisure` correctly and respects the minimum bound (0.0).

## 8. Questions
- How is a Job's "Prestige" defined? Statically per JobType, or dynamically based on colony priorities?
- *Architect:* Statically defined per `JobType` for the MVP (e.g., Doctors/Engineers have higher prestige than Haulers).
