#![allow(clippy::type_complexity)]
//! Radioactive Rats (Nova Feature).
//!
//! # The Spark
//! We have a `RadiationGrid` and `VerminState`. Vermin consume waste.
//!
//! # The Feature
//! What if `Toxic` vermin (acquired from consuming waste) also act as mobile radiation sources?
//! This expands the `VerminTrait::Toxic` to not only resist pest control but also actively pollute the `RadiationGrid`.
//!
//! # The Potential
//! Connects the environment hazard (radiation) with the dynamic entity system (vermin).
//! Players can't just wall off a radioactive spill anymore; they must actively exterminate vermin
//! before they carry the radiation into the colony's living quarters.

use crate::layer1::entities::vermin::{VerminState, VerminTrait};
use crate::layer1::nature::radioactive::RadiationGrid;
use bevy_ecs::prelude::*;
use rand::Rng;

const RADIATED_VERMIN_SOURCE_INTENSITY: f32 = 2.0;
const RADIATED_VERMIN_SOURCE_RADIUS: f32 = 2.0;

/// System to allow Toxic vermin to spread radiation randomly across the grid.
pub fn radioactive_vermin_spread_system(
    vermin_state: Option<Res<VerminState>>,
    radiation_grid: Option<ResMut<RadiationGrid>>,
    mut log: Option<ResMut<crate::shared::log::MessageLog>>,
) {
    let (Some(vermin), Some(mut grid)) = (vermin_state, radiation_grid) else {
        return;
    };

    // Only apply if the vermin are Toxic and have some severity
    if vermin.severity < 5.0 || !vermin.traits.contains(&VerminTrait::Toxic) {
        return;
    }

    let mut rng = rand::thread_rng();

    // The chance to drop a radiation spot scales with severity
    // At severity 100, 10% chance per tick. At severity 10, 1% chance.
    let chance = vermin.severity / 1000.0;

    if rng.gen::<f32>() < chance {
        // Pick a random location on the grid to simulate a vermin dropping radioactive waste
        let x = rng.gen_range(0..grid.width) as i32;
        let y = rng.gen_range(0..grid.height) as i32;

        grid.add_source(
            x,
            y,
            RADIATED_VERMIN_SOURCE_INTENSITY,
            RADIATED_VERMIN_SOURCE_RADIUS,
        );

        if let Some(ref mut l) = log {
            // Only log rarely so we don't spam
            if rng.gen::<f32>() < 0.1 {
                l.add_colored(
                    format!(
                        "A glowing, toxic rat scurried by, leaving a radioactive trail at ({}, {})!",
                        x, y
                    ),
                    ratatui::style::Color::LightGreen,
                );
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(radioactive_vermin_spread_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_toxic_vermin_spread_radiation() {
        let mut world = World::new();

        world.insert_resource(RadiationGrid::new(20, 20));

        let mut vermin = VerminState {
            severity: 100.0,
            ..Default::default()
        };
        vermin.traits.insert(VerminTrait::Toxic);
        world.insert_resource(vermin);

        // Run the system many times to guarantee a trigger due to RNG
        for _ in 0..100 {
            world
                .run_system_once(radioactive_vermin_spread_system)
                .unwrap();
        }

        let grid = world.get_resource::<RadiationGrid>().unwrap();
        let mut total_radiation = 0.0;
        for &val in &grid.values {
            total_radiation += val;
        }

        assert!(
            total_radiation > 0.0,
            "Toxic vermin should have spread radiation"
        );
    }

    #[test]
    fn test_non_toxic_vermin_do_not_spread_radiation() {
        let mut world = World::new();

        world.insert_resource(RadiationGrid::new(20, 20));

        let vermin = VerminState {
            severity: 100.0,
            ..Default::default()
        };
        world.insert_resource(vermin);

        for _ in 0..100 {
            world
                .run_system_once(radioactive_vermin_spread_system)
                .unwrap();
        }

        let grid = world.get_resource::<RadiationGrid>().unwrap();
        let mut total_radiation = 0.0;
        for &val in &grid.values {
            total_radiation += val;
        }

        assert_eq!(
            total_radiation, 0.0,
            "Non-toxic vermin should NOT spread radiation"
        );
    }
}
