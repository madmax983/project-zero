#![allow(clippy::type_complexity)]
//! Sleepwalking Hazards (Nova Feature).
//!
//! # The Spark
//! We have a `Sleepwalking` mental break (`ActionType::Sleepwalking`), a `TemperatureGrid` and `RadiationGrid`.
//!
//! # The Feature
//! What if a sleepwalking Pop could accidentally wander into extreme temperatures or highly radioactive zones, suffering damage or sickness before waking up?
//! This adds immediate physical danger to a psychological breakdown, forcing players to secure hazardous areas with physical barriers or `AccessControl` rather than just relying on Pops to pathfind around them intelligently.

use crate::layer1::biology::health::Health;
use crate::layer1::map::GridPosition;
use crate::layer1::mind::utility_types::{ActionType, PopAction};
use crate::layer1::nature::radioactive::RadiationGrid;
use crate::layer1::nature::radioactive::RadiationSickness;
use crate::layer1::nature::temperature::TemperatureGrid;
use crate::layer1::pop::Pop;
use bevy_ecs::prelude::*;

const SLEEPWALKING_HEAT_DAMAGE: f32 = 0.5;
const SLEEPWALKING_COLD_DAMAGE: f32 = 0.5;
const SLEEPWALKING_RADIATION_EXPOSURE: f32 = 2.0;

/// System that applies hazards to sleepwalking pops.
pub fn sleepwalking_hazards_system(
    mut commands: Commands,
    temperature_grid: Option<Res<TemperatureGrid>>,
    radiation_grid: Option<Res<RadiationGrid>>,
    mut pops: Query<
        (
            Entity,
            &GridPosition,
            &PopAction,
            &mut Health,
            Option<&mut RadiationSickness>,
        ),
        With<Pop>,
    >,
    mut log: Option<ResMut<crate::shared::log::MessageLog>>,
) {
    for (entity, pos, action, mut health, mut sickness_opt) in pops.iter_mut() {
        if action.current == ActionType::Sleepwalking {
            // Check Temperature Hazard
            if let Some(ref temp_grid) = temperature_grid {
                let temp = temp_grid.get(pos.x as usize, pos.y as usize);
                if temp > 50.0 {
                    health.current -= SLEEPWALKING_HEAT_DAMAGE;
                    if let Some(ref mut l) = log {
                        l.add_colored(
                            format!(
                                "A sleepwalking pop wandered into extreme heat at ({}, {})!",
                                pos.x, pos.y
                            ),
                            ratatui::style::Color::Red,
                        );
                    }
                } else if temp < -20.0 {
                    health.current -= SLEEPWALKING_COLD_DAMAGE;
                    if let Some(ref mut l) = log {
                        l.add_colored(
                            format!(
                                "A sleepwalking pop is freezing to death at ({}, {})!",
                                pos.x, pos.y
                            ),
                            ratatui::style::Color::Blue,
                        );
                    }
                }
            }

            // Check Radiation Hazard
            if let Some(ref rad_grid) = radiation_grid {
                let rad = rad_grid.get(pos.x as usize, pos.y as usize);
                if rad > 1.0 {
                    if let Some(ref mut sick) = sickness_opt {
                        sick.severity += SLEEPWALKING_RADIATION_EXPOSURE;
                    } else {
                        commands.entity(entity).insert(RadiationSickness {
                            severity: SLEEPWALKING_RADIATION_EXPOSURE,
                        });
                    }
                    if let Some(ref mut l) = log {
                        l.add_colored(
                            format!(
                                "A sleepwalking pop absorbed dangerous radiation at ({}, {})!",
                                pos.x, pos.y
                            ),
                            ratatui::style::Color::Magenta,
                        );
                    }
                }
            }

            if health.current <= 0.0 {
                // Ensure they don't go below 0
                health.current = 0.0;
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(sleepwalking_hazards_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_sleepwalking_pop_takes_heat_damage() {
        let mut world = World::new();

        let mut temp_grid = TemperatureGrid::new(10, 10, 0.0);
        temp_grid.set(5, 5, 100.0); // Extreme heat
        world.insert_resource(temp_grid);

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                PopAction {
                    current: ActionType::Sleepwalking,
                    ..Default::default()
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        world.run_system_once(sleepwalking_hazards_system).unwrap();

        let health = world.get::<Health>(pop).unwrap();
        assert!(health.current < 100.0, "Pop should take heat damage");
    }
}
