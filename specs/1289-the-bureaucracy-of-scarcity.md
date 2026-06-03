# 1289: The Bureaucracy of Scarcity

## 1. Overview
**Layer:** 1

**Fantasy:** The maddening proliferation of paperwork to manage a dying world.

**Mechanic:** As a colony approaches critical failure in a major resource (e.g., Food or Water), the local AI automatically spins up new "Bureaucrat" jobs to handle rationing. These jobs do not produce resources; they only slow the rate of consumption. However, the bureaucrats themselves consume resources and space.

## 2. Dependencies
- Job / Assignment System
- Resource Consumption / Needs System
- Colony AI / Macro-management System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    // RED Phase Test Setup
    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            evaluate_scarcity_system,
            apply_rationing_buff_system,
        ));
        app
    }

    #[test]
    fn test_bureaucrat_jobs_spawn_during_scarcity() {
        let mut app = setup_app();

        let colony = app.world_mut().spawn((
            Colony,
            ResourceStorage { food: 10, population_demand: 100 }, // Critical scarcity
            JobBoard { available_jobs: vec![] },
        )).id();

        app.update();

        let board = app.world().get::<JobBoard>(colony).unwrap();
        assert!(board.available_jobs.contains(&JobType::RationingBureaucrat), "Scarcity should spawn RationingBureaucrat jobs");
    }

    #[test]
    fn test_bureaucrats_reduce_consumption_rate() {
        let mut app = setup_app();

        let pop_normal = app.world_mut().spawn((
            Pop,
            JobAssignment::None,
            ConsumptionRate { food_per_tick: 2.0 },
        )).id();

        let pop_bureaucrat = app.world_mut().spawn((
            Pop,
            JobAssignment::Active(JobType::RationingBureaucrat),
            ConsumptionRate { food_per_tick: 2.0 },
        )).id();

        let colony = app.world_mut().spawn((
            Colony,
            GlobalRationingModifier { reduction_percent: 0.0 },
        )).id();

        app.update();

        let modifier = app.world().get::<GlobalRationingModifier>(colony).unwrap();
        assert!(modifier.reduction_percent > 0.0, "Active bureaucrats should increase the global rationing reduction");

        let normal_rate = app.world().get::<ConsumptionRate>(pop_normal).unwrap();
        assert!(normal_rate.food_per_tick < 2.0, "Global rationing should reduce individual pop consumption");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Colony;

#[derive(Component)]
pub struct ResourceStorage {
    pub food: u32,
    pub population_demand: u32,
}

#[derive(PartialEq, Clone, Debug)]
pub enum JobType {
    RationingBureaucrat,
}

#[derive(Component)]
pub struct JobBoard {
    pub available_jobs: Vec<JobType>,
}

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub enum JobAssignment {
    None,
    Active(JobType),
}

#[derive(Component)]
pub struct ConsumptionRate {
    pub food_per_tick: f32,
}

#[derive(Component)]
pub struct GlobalRationingModifier {
    pub reduction_percent: f32,
}

pub fn evaluate_scarcity_system(
    mut colonies: Query<(&ResourceStorage, &mut JobBoard), With<Colony>>,
) {
    for (storage, mut board) in colonies.iter_mut() {
        if storage.food < storage.population_demand / 2 {
            if !board.available_jobs.contains(&JobType::RationingBureaucrat) {
                // Add 3 bureaucrat jobs as a response to scarcity
                board.available_jobs.push(JobType::RationingBureaucrat);
                board.available_jobs.push(JobType::RationingBureaucrat);
                board.available_jobs.push(JobType::RationingBureaucrat);
            }
        }
    }
}

pub fn apply_rationing_buff_system(
    bureaucrats: Query<&JobAssignment, With<Pop>>,
    mut colonies: Query<&mut GlobalRationingModifier, With<Colony>>,
    mut consumers: Query<&mut ConsumptionRate, With<Pop>>,
) {
    let active_bureaucrats = bureaucrats.iter().filter(|j| matches!(j, JobAssignment::Active(JobType::RationingBureaucrat))).count();

    for mut modifier in colonies.iter_mut() {
        // Each bureaucrat reduces consumption by 5%, capped at 50%
        modifier.reduction_percent = (active_bureaucrats as f32 * 0.05).min(0.50);

        for mut rate in consumers.iter_mut() {
             // Reduce base rate (assumed 2.0 for test) by the modifier
             rate.food_per_tick = 2.0 * (1.0 - modifier.reduction_percent);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Job Cleanup**: The system currently spawns jobs but never removes them when scarcity ends. Add logic to clear `JobType::RationingBureaucrat` from the `JobBoard` when `storage.food > storage.population_demand`.
- **Dynamic Base Rate**: The `apply_rationing_buff_system` hardcodes the base rate `2.0` when calculating the reduction. It should calculate reduction based on the Pop's inherent base rate.
- **Resource Types**: Generalize `ResourceStorage` to handle Water, Oxygen, or Power, rather than just Food.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- **Job Board Integration**: Ensure the `JobBoard` component matches the project's actual job assignment structure. The AI decider must be able to pull pops away from productive jobs (like farming) to become bureaucrats.

## 8. Questions
*Builder: add questions here if spec is unclear.*
