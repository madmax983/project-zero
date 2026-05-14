use bevy_ecs::prelude::*;
use crate::layer2::governance::{Governor, GovernorStats};

#[derive(Component)]
pub struct ColonyState {
    pub food_reserves: u32,
    pub unrest: f32,
}

#[derive(Component)]
pub struct ColonyReport {
    pub reported_food: u32,
    pub reported_unrest: f32,
}

#[derive(Component)]
pub struct Inquisitor;

fn calculate_falsified_report(state: &ColonyState, stats: &GovernorStats) -> ColonyReport {
    let is_corrupt = stats.corruption > 80.0 || stats.loyalty < 20.0;
    let corruption_factor = (stats.corruption / 100.0).clamp(0.0, 1.0);

    if is_corrupt {
        // Highly corrupt acts like 100% or more, but let's make it drastic
        ColonyReport {
            reported_food: state.food_reserves + (state.food_reserves as f32 * 10.0) as u32, // 10x
            reported_unrest: 0.0,
        }
    } else {
        // Progressive falsification
        ColonyReport {
            reported_food: state.food_reserves + (state.food_reserves as f32 * corruption_factor * 2.0) as u32,
            reported_unrest: state.unrest * (1.0 - corruption_factor),
        }
    }
}

pub fn generate_colony_reports_system(
    mut commands: Commands,
    colonies: Query<(Entity, &ColonyState, &Governor, Option<&Inquisitor>)>,
    pop_stats: Query<&GovernorStats>,
) {
    for (entity, state, governor, inquisitor) in colonies.iter() {
        let report = if inquisitor.is_some() {
            ColonyReport {
                reported_food: state.food_reserves,
                reported_unrest: state.unrest,
            }
        } else if let Ok(stats) = pop_stats.get(governor.pop_entity) {
            calculate_falsified_report(state, stats)
        } else {
            // Missing governor stats, report truth
            ColonyReport {
                reported_food: state.food_reserves,
                reported_unrest: state.unrest,
            }
        };

        commands.entity(entity).insert(report);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::{App, Update};

    #[test]
    fn test_corrupt_governor_falsifies_colony_report() {
        let mut app = App::new();

        let pop_entity = app.world_mut().spawn(GovernorStats {
            loyalty: 10.0,
            corruption: 90.0,
            ambition: 0.0,
        }).id();

        let colony_entity = app.world_mut().spawn((
            ColonyState {
                food_reserves: 10,
                unrest: 80.0,
            },
            Governor {
                pop_entity,
                assigned_at: 0,
            }
        )).id();

        app.add_systems(Update, generate_colony_reports_system);
        app.update();

        let report = app.world().get::<ColonyReport>(colony_entity).unwrap();

        assert!(report.reported_food > 100, "Corrupt governor should falsely report high food");
        assert!(report.reported_unrest < 10.0, "Corrupt governor should falsely report low unrest");
    }

    #[test]
    fn test_loyal_governor_reports_truth() {
        let mut app = App::new();

        let pop_entity = app.world_mut().spawn(GovernorStats {
            loyalty: 90.0,
            corruption: 0.0,
            ambition: 0.0,
        }).id();

        let colony_entity = app.world_mut().spawn((
            ColonyState {
                food_reserves: 10,
                unrest: 80.0,
            },
            Governor {
                pop_entity,
                assigned_at: 0,
            }
        )).id();

        app.add_systems(Update, generate_colony_reports_system);
        app.update();

        let report = app.world().get::<ColonyReport>(colony_entity).unwrap();

        assert_eq!(report.reported_food, 10, "Loyal governor should report exact food");
        assert_eq!(report.reported_unrest, 80.0, "Loyal governor should report exact unrest");
    }

    #[test]
    fn test_progressive_falsification_scales_with_corruption() {
        let mut app = App::new();

        let pop_entity = app.world_mut().spawn(GovernorStats {
            loyalty: 50.0,
            corruption: 50.0,
            ambition: 0.0,
        }).id();

        let colony_entity = app.world_mut().spawn((
            ColonyState {
                food_reserves: 100,
                unrest: 50.0,
            },
            Governor {
                pop_entity,
                assigned_at: 0,
            }
        )).id();

        app.add_systems(Update, generate_colony_reports_system);
        app.update();

        let report = app.world().get::<ColonyReport>(colony_entity).unwrap();

        // With 50% corruption, report should be halfway falsified
        assert!(report.reported_food > 100, "Should be partially falsified up");
        assert!(report.reported_food < 1000, "Should not be fully falsified");

        assert!(report.reported_unrest < 50.0, "Should be partially falsified down");
        assert!(report.reported_unrest > 0.0, "Should not be fully falsified");
    }

    #[test]
    fn test_inquisitor_reveals_truth() {
        let mut app = App::new();

        let pop_entity = app.world_mut().spawn(GovernorStats {
            loyalty: 0.0,
            corruption: 100.0,
            ambition: 0.0,
        }).id();

        let colony_entity = app.world_mut().spawn((
            ColonyState {
                food_reserves: 10,
                unrest: 80.0,
            },
            Governor {
                pop_entity,
                assigned_at: 0,
            },
            Inquisitor
        )).id();

        app.add_systems(Update, generate_colony_reports_system);
        app.update();

        let report = app.world().get::<ColonyReport>(colony_entity).unwrap();

        // Despite 100% corruption, Inquisitor forces the truth
        assert_eq!(report.reported_food, 10);
        assert_eq!(report.reported_unrest, 80.0);
    }
}
