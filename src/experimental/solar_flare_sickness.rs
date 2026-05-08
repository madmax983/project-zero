#![allow(clippy::type_complexity)]
//! Solar Flare Sickness (Nova Feature).
//!
//! # The Spark
//! We have `SolarCycle` (Solar Maximum) and `RadiationSickness`.
//!
//! # The Feature
//! What if during a Solar Maximum, Pops who are outside (not under a `RoofGrid`) have a chance to develop `RadiationSickness` due to solar flares?
//!
//! # The Potential
//! Connects the energy system (Solar Cycles) to the medical system. Players get lots of free power during Solar Maximum, but must keep their Pops indoors or risk a medical crisis.

use crate::layer1::day_night::{DayNightCycle, TimeOfDay};
use crate::layer1::map::GridPosition;
use crate::layer1::nature::radioactive::RadiationSickness;
use crate::layer1::nature::solar::{SolarCycle, SolarCycleState};
use crate::layer1::physics::structural_integrity::RoofGrid;
use crate::layer1::pop::Pop;
use bevy_ecs::prelude::*;
use rand::Rng;

const SOLAR_FLARE_SICKNESS_CHANCE: f64 = 0.05; // 5% chance per tick
const SOLAR_FLARE_EXPOSURE: f32 = 1.5; // Amount of sickness added

/// System that applies solar flare radiation to pops caught outside during a Solar Maximum day.
pub fn solar_flare_sickness_system(
    mut commands: Commands,
    solar_state: Option<Res<SolarCycleState>>,
    day_night: Option<Res<DayNightCycle>>,
    roof_grid: Option<Res<RoofGrid>>,
    mut pops: Query<(Entity, &GridPosition, Option<&mut RadiationSickness>), With<Pop>>,
    mut log: Option<ResMut<crate::shared::log::MessageLog>>,
) {
    // Only apply if it's Solar Maximum
    if let Some(state) = solar_state {
        if state.current_cycle != SolarCycle::Maximum {
            return;
        }
    } else {
        return;
    }

    // Only apply during the Day
    if let Some(dn) = day_night {
        if dn.time_of_day != TimeOfDay::Day {
            return;
        }
    } else {
        return;
    }

    let Some(ref grid) = roof_grid else { return };

    let mut rng = rand::thread_rng();

    for (entity, pos, sickness_opt) in pops.iter_mut() {
        // Check if pop is outside (no roof)
        if !grid.has_roof(pos.x, pos.y) && rng.gen_bool(SOLAR_FLARE_SICKNESS_CHANCE) {
            if let Some(mut sick) = sickness_opt {
                sick.severity = (sick.severity + SOLAR_FLARE_EXPOSURE).min(100.0);
            } else {
                commands.entity(entity).insert(RadiationSickness {
                    severity: SOLAR_FLARE_EXPOSURE,
                });
            }
            if let Some(ref mut l) = log {
                l.add_colored(
                    format!(
                        "A pop suffered radiation sickness from a solar flare at ({}, {})!",
                        pos.x, pos.y
                    ),
                    ratatui::style::Color::Magenta,
                );
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(solar_flare_sickness_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_solar_flare_sickness_outdoors() {
        let mut world = World::new();

        world.insert_resource(SolarCycleState {
            current_cycle: SolarCycle::Maximum,
        });
        world.insert_resource(DayNightCycle {
            time_of_day: TimeOfDay::Day,
            ..Default::default()
        });

        let grid = RoofGrid::new(10, 10);
        // No roof at (5, 5)
        world.insert_resource(grid);

        let pop = world.spawn((Pop, GridPosition { x: 5, y: 5 })).id();

        // Run multiple times to trigger the 5% chance reliably
        for _ in 0..100 {
            world.run_system_once(solar_flare_sickness_system).unwrap();
            // Apply deferred to ensure commands execute (inserting component)
            world.flush();
        }

        let sickness = world.get::<RadiationSickness>(pop);
        assert!(
            sickness.is_some(),
            "Pop outside during solar maximum day should get radiation sickness"
        );
    }

    #[test]
    fn test_no_solar_flare_sickness_indoors() {
        let mut world = World::new();

        world.insert_resource(SolarCycleState {
            current_cycle: SolarCycle::Maximum,
        });
        world.insert_resource(DayNightCycle {
            time_of_day: TimeOfDay::Day,
            ..Default::default()
        });

        let mut grid = RoofGrid::new(10, 10);
        // Roof at (5, 5)
        grid.set(5, 5, true);
        world.insert_resource(grid);

        let pop = world.spawn((Pop, GridPosition { x: 5, y: 5 })).id();

        for _ in 0..100 {
            world.run_system_once(solar_flare_sickness_system).unwrap();
            world.flush();
        }

        let sickness = world.get::<RadiationSickness>(pop);
        assert!(
            sickness.is_none(),
            "Pop inside should be protected from solar flares"
        );
    }
}
