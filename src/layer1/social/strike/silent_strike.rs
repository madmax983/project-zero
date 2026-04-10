use bevy_ecs::prelude::*;

use crate::layer1::pop::Pop;
#[allow(unused_imports)]
use crate::layer1::pop::Job;
use crate::layer1::social::morale::Morale;

/// Configuration for the silent strike mechanic.
#[derive(Resource)]
pub struct SilentStrikeConfig {
    /// Morale threshold below which a pop may go on silent strike.
    pub morale_threshold: f32,
}

impl Default for SilentStrikeConfig {
    fn default() -> Self {
        Self {
            morale_threshold: 0.2, // 20% morale
        }
    }
}

/// The state of a pop engaging in a silent strike.
/// While this is active, the pop consumes resources normally but `calculate_work_amount`
/// outputs 0.0 work per tick.
#[derive(Component)]
pub struct SilentStrike;

/// Evaluates if pops should enter a silent strike.
///
/// A silent strike occurs when a pop's morale is extremely low. They still show up for work
/// but produce 0 output, effectively occupying a job slot and consuming resources without contributing.
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::type_complexity)]
pub fn evaluate_silent_strike_system(
    mut commands: Commands,
    config: Option<Res<SilentStrikeConfig>>,
    query: Query<(Entity, &Morale, Option<&SilentStrike>), With<Pop>>,
) {
    let default_config = SilentStrikeConfig::default();
    let config = config.as_deref().unwrap_or(&default_config);

    for (entity, morale, strike_opt) in query.iter() {
        let should_strike = morale.value < config.morale_threshold;

        if should_strike {
            if strike_opt.is_none() {
                commands.entity(entity).insert(SilentStrike);
            }
        } else if strike_opt.is_some() {
            // Conditions improved, end the strike
            commands.entity(entity).remove::<SilentStrike>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_silent_strike_activation() {
        let mut world = World::new();

        let entity = world
            .spawn((
                Pop,
                Morale {
                    value: 0.1,
                    ..Default::default()
                },
                Job {
                    workplace: Entity::PLACEHOLDER,
                    job_type: crate::layer1::actions::AssignmentType::FarmWorker,
                },
            ))
            .id();

        world.run_system_once(evaluate_silent_strike_system).unwrap();

        // Should gain the SilentStrike component
        assert!(world.get::<SilentStrike>(entity).is_some());
    }

    #[test]
    fn test_silent_strike_resolution() {
        let mut world = World::new();

        let entity = world
            .spawn((
                Pop,
                SilentStrike,
                Morale {
                    value: 0.5,
                    ..Default::default()
                },
                Job {
                    workplace: Entity::PLACEHOLDER,
                    job_type: crate::layer1::actions::AssignmentType::FarmWorker,
                },
            ))
            .id();

        world.run_system_once(evaluate_silent_strike_system).unwrap();

        // Should lose the SilentStrike component
        assert!(world.get::<SilentStrike>(entity).is_none());
    }

    #[test]
    fn test_silent_strike_production_halt() {
        let mut world = World::new();

        let pop_striking = world.spawn((Pop, SilentStrike)).id();
        let pop_working = world.spawn((Pop,)).id();

        let amount_striking = crate::layer1::execution::calculate_work_amount(
            &world,
            pop_striking,
            crate::layer1::designation::DesignationType::Mine,
            None,
            0.5,
            1.0,
            1.0,
        );

        let amount_working = crate::layer1::execution::calculate_work_amount(
            &world,
            pop_working,
            crate::layer1::designation::DesignationType::Mine,
            None,
            0.5,
            1.0,
            1.0,
        );

        assert_eq!(
            amount_striking, 0.0,
            "Silent strike should produce 0 work output."
        );
        assert!(
            amount_working > 0.0,
            "Normal shift should produce work output."
        );
    }
}
