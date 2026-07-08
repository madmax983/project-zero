//! Piezoelectric Batteries System.
//!
//! A Nova experimental feature that charges batteries passively
//! based on the ambient tectonic stress in the geology layer.

use crate::layer1::energy::Battery;
use crate::layer1::geology::tectonic::TectonicStress;
use bevy_ecs::prelude::*;

/// System that applies a charge to batteries based on TectonicStress.
/// If stress is above 50% of the threshold, it produces free power.
pub fn piezoelectric_batteries_system(
    stress: Option<Res<TectonicStress>>,
    mut batteries: Query<&mut Battery>,
) {
    if let Some(stress) = stress {
        // High tectonic stress (e.g. nearing a megaquake) will slowly charge all batteries.
        let charge_threshold = stress.threshold * 0.5;
        if stress.current > charge_threshold {
            let charge_amount = (stress.current - charge_threshold) * 0.05;
            for mut battery in batteries.iter_mut() {
                battery.charge(charge_amount);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_piezoelectric_charging_high_stress() {
        let mut world = World::new();

        world.insert_resource(TectonicStress {
            current: 80.0,
            threshold: 100.0,
            dissipation_rate: 0.1,
        });

        let battery_entity = world
            .spawn(Battery {
                capacity: 100.0,
                charge: 10.0,
                max_throughput: 5.0,
            })
            .id();

        let _ = world.run_system_once(piezoelectric_batteries_system);

        let battery = world.get::<Battery>(battery_entity).unwrap();
        // threshold is 50. charge_amount = (80.0 - 50.0) * 0.05 = 30.0 * 0.05 = 1.5
        assert_eq!(battery.charge, 11.5);
    }

    #[test]
    fn test_piezoelectric_no_charge_low_stress() {
        let mut world = World::new();

        world.insert_resource(TectonicStress {
            current: 20.0, // below 50.0
            threshold: 100.0,
            dissipation_rate: 0.1,
        });

        let battery_entity = world
            .spawn(Battery {
                capacity: 100.0,
                charge: 10.0,
                max_throughput: 5.0,
            })
            .id();

        let _ = world.run_system_once(piezoelectric_batteries_system);

        let battery = world.get::<Battery>(battery_entity).unwrap();
        assert_eq!(battery.charge, 10.0);
    }
}
