use crate::layer1::justice::Inmate;
use crate::layer1::map::GridPosition;
use crate::layer1::zone::{ZoneGrid, ZoneType};
use bevy_ecs::prelude::*;

/// Component indicating an Inmate is currently performing forced labor.
///
/// This component is added by [`evaluate_penal_work_system`] when an Inmate is in a [`ZoneType::Penal`],
/// and removed by [`cleanup_penal_work_system`] when they leave.
#[derive(Component, Clone, Copy, Debug)]
pub struct PenalLabor {
    /// Work efficiency multiplier (default > 1.0 because fear motivates).
    pub efficiency_bonus: f32,
}

impl Default for PenalLabor {
    fn default() -> Self {
        Self {
            efficiency_bonus: 0.2, // +20% speed
        }
    }
}

/// Tracks the risk of an inmate rebelling.
///
/// Accumulates over time while working.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct RevoltRisk {
    /// Current risk level.
    pub current: f32,
    /// Threshold at which a jailbreak occurs.
    pub threshold: f32,
}

/// Checks if Inmates are in a Penal Zone and assigns PenalLabor status.
pub fn evaluate_penal_work_system(
    mut commands: Commands,
    zone_grid: Res<ZoneGrid>,
    query: Query<(Entity, &GridPosition), (With<Inmate>, Without<PenalLabor>)>,
) {
    for (entity, pos) in &query {
        if zone_grid.get(pos.x, pos.y) == ZoneType::Penal {
            commands.entity(entity).insert((
                PenalLabor::default(),
                RevoltRisk {
                    current: 0.0,
                    threshold: 100.0,
                },
            ));
        }
    }
}

/// Removes PenalLabor if Inmate leaves zone.
pub fn cleanup_penal_work_system(
    mut commands: Commands,
    zone_grid: Res<ZoneGrid>,
    query: Query<(Entity, &GridPosition), With<PenalLabor>>,
) {
    for (entity, pos) in &query {
        if zone_grid.get(pos.x, pos.y) != ZoneType::Penal {
            commands
                .entity(entity)
                .remove::<PenalLabor>()
                .remove::<RevoltRisk>();
        }
    }
}

/// Increases revolt risk for working inmates.
pub fn update_revolt_risk_system(mut query: Query<&mut RevoltRisk, With<PenalLabor>>) {
    for mut risk in &mut query {
        risk.current += 0.1;
    }
}

/// Triggers jailbreak if risk exceeds threshold.
pub fn check_jailbreak_system(
    mut commands: Commands,
    query: Query<(Entity, &RevoltRisk), With<Inmate>>,
) {
    for (entity, risk) in &query {
        if risk.current >= risk.threshold {
            commands
                .entity(entity)
                .remove::<Inmate>()
                .remove::<PenalLabor>()
                .remove::<RevoltRisk>();

            // TODO: Add Wanted status or aggression
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::justice::Inmate;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::utility_types::PopAction;
    use crate::layer1::zone::{ZoneGrid, ZoneType};

    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        let mut zone_grid = ZoneGrid::new(10, 10);
        // Define a Penal Zone at (5,5)
        zone_grid.set(5, 5, ZoneType::Penal);
        world.insert_resource(zone_grid);
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world
    }

    #[test]
    fn test_penal_labor_component_defaults() {
        let labor = PenalLabor::default();
        assert!((labor.efficiency_bonus - 0.2).abs() < f32::EPSILON); // 20% faster
    }

    #[test]
    fn test_inmate_evaluates_work_in_penal_zone() {
        let mut world = setup_world();

        // Inmate at (5,5) which is a Penal Zone
        let inmate = world
            .spawn((
                Pop,
                Inmate {
                    sentence_ticks: 1000,
                },
                GridPosition { x: 5, y: 5 },
                PopAction::default(),
            ))
            .id();

        // Run evaluation system
        world.run_system_once(evaluate_penal_work_system).unwrap();

        // Should receive PenalLabor component
        assert!(world.get::<PenalLabor>(inmate).is_some());
        // Should receive RevoltRisk
        assert!(world.get::<RevoltRisk>(inmate).is_some());
    }

    #[test]
    fn test_inmate_outside_penal_zone_is_idle() {
        let mut world = setup_world();

        // Inmate at (0,0) - NOT a Penal Zone
        let inmate = world
            .spawn((
                Pop,
                Inmate {
                    sentence_ticks: 1000,
                },
                GridPosition { x: 0, y: 0 },
                PopAction::default(),
            ))
            .id();

        // Ensure system doesn't wrongly add it
        world.run_system_once(evaluate_penal_work_system).unwrap();
        assert!(world.get::<PenalLabor>(inmate).is_none());

        // Test cleanup
        world
            .entity_mut(inmate)
            .insert((PenalLabor::default(), RevoltRisk::default()));

        // Still at (0,0) which is not Penal
        world.run_system_once(cleanup_penal_work_system).unwrap();

        assert!(world.get::<PenalLabor>(inmate).is_none());
    }

    #[test]
    fn test_working_accumulates_revolt_risk() {
        let mut world = setup_world();

        let inmate = world
            .spawn((
                Pop,
                Inmate {
                    sentence_ticks: 1000,
                },
                PenalLabor::default(), // Actively working
                RevoltRisk {
                    current: 0.0,
                    threshold: 100.0,
                },
            ))
            .id();

        // Run system tick
        world.run_system_once(update_revolt_risk_system).unwrap();

        let risk = world.get::<RevoltRisk>(inmate).unwrap();
        assert!(risk.current > 0.0);
    }

    #[test]
    fn test_jailbreak_trigger() {
        let mut world = setup_world();

        let inmate = world
            .spawn((
                Pop,
                Inmate {
                    sentence_ticks: 1000,
                },
                RevoltRisk {
                    current: 101.0,
                    threshold: 100.0,
                }, // Over threshold
            ))
            .id();

        world.run_system_once(check_jailbreak_system).unwrap();

        // Should lose Inmate status (escaped)
        assert!(world.get::<Inmate>(inmate).is_none());
        assert!(world.get::<PenalLabor>(inmate).is_none());
        assert!(world.get::<RevoltRisk>(inmate).is_none());
    }
}
