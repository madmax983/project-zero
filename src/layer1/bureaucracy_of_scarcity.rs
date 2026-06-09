//! The Bureaucracy of Scarcity
//!
//! "When the granaries are full, the people sing. When they are empty, we count the grains."
//!
//! This module simulates the administrative response to resource shortages. When the colony's
//! food supplies drop below a critical threshold, emergency bureaucratic jobs are created.
//! These [`JobType::RationingBureaucrat`] positions employ pops to strictly enforce consumption limits,
//! reducing the overall food drain on the colony at the cost of labor that could be used elsewhere.

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
    pub base_food_per_tick: f32,
    pub food_per_tick: f32,
}

#[derive(Component)]
pub struct GlobalRationingModifier {
    pub reduction_percent: f32,
}

/// Evaluates the colony's resource storage against its population demand.
///
/// **Why it exists:** To detect starvation spirals before they happen. If food reserves
/// fall below 50% of what the population demands, this system automatically intervenes
/// by posting emergency [`JobType::RationingBureaucrat`] openings to the [`JobBoard`].
///
/// # Examples
///
/// ```
/// use bevy::prelude::*;
/// use scale::layer1::bureaucracy_of_scarcity::*;
///
/// let mut app = App::new();
/// app.add_systems(Update, evaluate_scarcity_system);
///
/// // Create a starving colony
/// let colony = app.world_mut().spawn((
///     Colony,
///     ResourceStorage { food: 10, population_demand: 100 },
///     JobBoard { available_jobs: vec![] }
/// )).id();
///
/// app.update();
///
/// // The system panicked and created bureaucratic jobs
/// let board = app.world().get::<JobBoard>(colony).unwrap();
/// assert!(board.available_jobs.contains(&JobType::RationingBureaucrat));
/// ```
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
        } else if storage.food > storage.population_demand {
            board
                .available_jobs
                .retain(|job| *job != JobType::RationingBureaucrat);
        }
    }
}

/// Applies a global rationing modifier based on the number of active bureaucrats.
///
/// **Why it exists:** To mechanically simulate the effect of the bureaucracy.
/// Every employed rationing bureaucrat reduces the colony's global food consumption
/// by 5% (up to a maximum of 50%). This creates a survival mechanic where labor
/// is traded for efficiency during famines.
///
/// # Examples
///
/// ```
/// use bevy::prelude::*;
/// use scale::layer1::bureaucracy_of_scarcity::*;
///
/// let mut app = App::new();
/// app.add_systems(Update, apply_rationing_buff_system);
///
/// // Spawn an active bureaucrat
/// app.world_mut().spawn((Pop, JobAssignment::Active(JobType::RationingBureaucrat)));
///
/// // Spawn a normal consumer
/// let consumer = app.world_mut().spawn((
///     Pop,
///     JobAssignment::None,
///     ConsumptionRate { base_food_per_tick: 2.0, food_per_tick: 2.0 }
/// )).id();
///
/// // Spawn the colony tracker
/// let colony = app.world_mut().spawn((
///     Colony,
///     GlobalRationingModifier { reduction_percent: 0.0 }
/// )).id();
///
/// app.update();
///
/// // The active bureaucrat applied a 5% reduction
/// let modifier = app.world().get::<GlobalRationingModifier>(colony).unwrap();
/// assert_eq!(modifier.reduction_percent, 0.05);
///
/// // The consumer's actual rate is reduced
/// let rate = app.world().get::<ConsumptionRate>(consumer).unwrap();
/// assert!(rate.food_per_tick < rate.base_food_per_tick);
/// ```
pub fn apply_rationing_buff_system(
    bureaucrats: Query<&JobAssignment, With<Pop>>,
    mut colonies: Query<&mut GlobalRationingModifier, With<Colony>>,
    mut consumers: Query<&mut ConsumptionRate, With<Pop>>,
) {
    let active_bureaucrats = bureaucrats
        .iter()
        .filter(|j| matches!(j, JobAssignment::Active(JobType::RationingBureaucrat)))
        .count();

    for mut modifier in colonies.iter_mut() {
        // Each bureaucrat reduces consumption by 5%, capped at 50%
        modifier.reduction_percent = (active_bureaucrats as f32 * 0.05).min(0.50);

        for mut rate in consumers.iter_mut() {
            // Reduce base rate by the modifier
            rate.food_per_tick = rate.base_food_per_tick * (1.0 - modifier.reduction_percent);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED Phase Test Setup
    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(
            Update,
            (evaluate_scarcity_system, apply_rationing_buff_system),
        );
        app
    }

    #[test]
    fn test_bureaucrat_jobs_spawn_during_scarcity() {
        let mut app = setup_app();

        let colony = app
            .world_mut()
            .spawn((
                Colony,
                ResourceStorage {
                    food: 10,
                    population_demand: 100,
                }, // Critical scarcity
                JobBoard {
                    available_jobs: vec![],
                },
            ))
            .id();

        app.update();

        let board = app.world().get::<JobBoard>(colony).unwrap();
        assert!(
            board.available_jobs.contains(&JobType::RationingBureaucrat),
            "Scarcity should spawn RationingBureaucrat jobs"
        );
    }

    #[test]
    fn test_bureaucrats_reduce_consumption_rate() {
        let mut app = setup_app();

        let pop_normal = app
            .world_mut()
            .spawn((
                Pop,
                JobAssignment::None,
                ConsumptionRate {
                    base_food_per_tick: 2.0,
                    food_per_tick: 2.0,
                },
            ))
            .id();

        let _pop_bureaucrat = app
            .world_mut()
            .spawn((
                Pop,
                JobAssignment::Active(JobType::RationingBureaucrat),
                ConsumptionRate {
                    base_food_per_tick: 2.0,
                    food_per_tick: 2.0,
                },
            ))
            .id();

        let colony = app
            .world_mut()
            .spawn((
                Colony,
                GlobalRationingModifier {
                    reduction_percent: 0.0,
                },
            ))
            .id();

        app.update();

        let modifier = app.world().get::<GlobalRationingModifier>(colony).unwrap();
        assert!(
            modifier.reduction_percent > 0.0,
            "Active bureaucrats should increase the global rationing reduction"
        );

        let normal_rate = app.world().get::<ConsumptionRate>(pop_normal).unwrap();
        assert!(
            normal_rate.food_per_tick < 2.0,
            "Global rationing should reduce individual pop consumption"
        );
    }
}
