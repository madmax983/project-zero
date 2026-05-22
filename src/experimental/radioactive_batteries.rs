//! Radioactive Batteries (Nova Feature).
//!
//! # The Spark
//! We have a `RadiationGrid` and `Battery` components. Radiation is currently just a hazard.
//!
//! # The Feature
//! What if a `Battery` placed in a highly radioactive zone slowly charges itself?
//! This system (`radioactive_batteries_system`) checks the `RadiationGrid` at a `Battery`'s location.
//! If radiation is present, it converts a fraction of that radiation into battery charge.
//!
//! # The Potential
//! Connects the environment hazard (radiation) with the energy system (batteries).
//! Players can build "dirty" power grids by intentionally storing nuclear waste next to
//! battery banks, risking `RadiationSickness` for Pops who walk near them in exchange for free passive power.

use crate::layer1::energy::Battery;
use crate::layer1::map::GridPosition;
use crate::layer1::nature::radioactive::RadiationGrid;
use bevy_ecs::prelude::*;

/// System that charges batteries based on ambient radiation.
pub fn radioactive_batteries_system(
    radiation_grid: Option<Res<RadiationGrid>>,
    mut batteries: Query<(&mut Battery, &GridPosition)>,
) {
    let Some(grid) = radiation_grid else {
        return;
    };

    for (mut battery, pos) in &mut batteries {
        let rad_level = grid.get(pos.x as usize, pos.y as usize);
        if rad_level > 0.0 {
            // Convert radiation to charge.
            // E.g. 5.0 rads -> 0.5 charge per tick.
            let charge_amount = (rad_level * 0.1).min(battery.max_throughput);
            battery.charge(charge_amount);
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(radioactive_batteries_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_radioactive_batteries_charge() {
        let mut world = World::new();

        let mut grid = RadiationGrid::new(10, 10);
        grid.set(5, 5, 10.0); // 10.0 rads at (5, 5)
        world.insert_resource(grid);

        let battery_entity = world
            .spawn((
                Battery {
                    capacity: 100.0,
                    charge: 0.0,
                    max_throughput: 5.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        world.run_system_once(radioactive_batteries_system).unwrap();

        let battery = world.get::<Battery>(battery_entity).unwrap();
        // 10.0 rads * 0.1 = 1.0 charge
        assert!(
            (battery.charge - 1.0).abs() < f32::EPSILON,
            "Battery should have 1.0 charge, got {}",
            battery.charge
        );
    }

    #[test]
    fn test_radioactive_batteries_no_radiation() {
        let mut world = World::new();

        world.insert_resource(RadiationGrid::new(10, 10)); // Empty grid

        let battery_entity = world
            .spawn((
                Battery {
                    capacity: 100.0,
                    charge: 0.0,
                    max_throughput: 5.0,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        world.run_system_once(radioactive_batteries_system).unwrap();

        let battery = world.get::<Battery>(battery_entity).unwrap();
        assert_eq!(
            battery.charge, 0.0,
            "Battery should not charge without radiation"
        );
    }

    #[test]
    fn test_radioactive_batteries_throughput_limit() {
        let mut world = World::new();

        let mut grid = RadiationGrid::new(10, 10);
        grid.set(5, 5, 100.0); // 100.0 rads -> 10.0 potential charge
        world.insert_resource(grid);

        let battery_entity = world
            .spawn((
                Battery {
                    capacity: 100.0,
                    charge: 0.0,
                    max_throughput: 2.0, // Limits charge to 2.0 per tick
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        world.run_system_once(radioactive_batteries_system).unwrap();

        let battery = world.get::<Battery>(battery_entity).unwrap();
        assert!(
            (battery.charge - 2.0).abs() < f32::EPSILON,
            "Battery charge should be limited by max_throughput, got {}",
            battery.charge
        );
    }
}
